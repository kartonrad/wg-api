use serde::{Serialize, Deserialize};
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey, get_current_timestamp, TokenData};
use actix_web::{ HttpResponse, Responder, get, http::{header::{self, ContentType}, StatusCode}, FromRequest, dev::{Payload, ConnectionInfo}, HttpRequest, error::InternalError, web,  post, cookie::Cookie};
#[allow(unused_imports)]
use log::{error, warn, info, debug, trace};
use lazy_static::lazy_static;
use serde_json::json;
use time::{PrimitiveDateTime, OffsetDateTime};
use super::DB_POOL;
use pbkdf2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Pbkdf2
};

use thiserror::Error as ThisErrorError;

use std::{env};

use actix_web::{
    Error,
};
use futures_util::{future::LocalBoxFuture, FutureExt};

// ================================================================================== CONSTS/STATICS ==================================================================================
const JWT_ALGO: Algorithm = Algorithm::HS256;
lazy_static! {
    static ref JWT_ISS: String = format!("{} v{} by {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"), env!("CARGO_PKG_AUTHORS")).to_string();
    static ref JWT_SECRET: String = env::var("JWT_SECRET").unwrap();
    static ref JWT_ENC_KEY : EncodingKey = EncodingKey::from_secret(JWT_SECRET.as_ref());
    static ref JWT_DEC_KEY : DecodingKey = DecodingKey::from_secret(JWT_SECRET.as_ref());
    static ref JWT_VAL : Validation = {
        let mut val =Validation::new(JWT_ALGO);
        val.set_issuer(&[JWT_ISS.to_string()]);
        val.leeway = 180; // 3 min        
        val
    };
}

// ================================================================================== STATE MODEL ==================================================================================
/// Our claims struct, it needs to derive `Serialize` and/or `Deserialize`
#[derive(Serialize,Deserialize)]
struct JWTClaims {
    auth_as: i32, // Id
    aud: String,         // Optional. Audience
    exp: u64,          // Required (validate_exp defaults to true in validation). Expiration time (as UTC timestamp)
    iat: u64,          // Optional. Issued at (as UTC timestamp)
    iss: String,         // Optional. Issuer
    // nbf: usize,          // Optional. Not Before (as UTC timestamp)
    //sub: String,         // Optional. Subject (whom token refers to)
}

#[derive(Debug, Serialize, Clone)]
#[serde(into = "SerdeIdentity")] 
pub struct Identity {
    pub id: i32,
    pub profile_pic: Option<i32>,
    pub name: String,
    pub bio: String,

    pub username: String, // '[abcdefghijklmopqrstuvwxyz0123456789-_]+'
    pub password_hash: String,
    pub revoke_before: time::PrimitiveDateTime,

    pub wg: Option<i32>
}
#[derive(Debug, Serialize, Deserialize)]
struct SerdeIdentity {
    id: i32,profile_pic: Option<i32>,name: String,bio: String,username: String,password_hash: String,wg: Option<i32>,
    #[serde(with = "time::serde::rfc3339")]
    revoke_before: time::OffsetDateTime
}
impl Into<SerdeIdentity> for Identity {
    fn into(self) -> SerdeIdentity {
        SerdeIdentity{ id: self.id, profile_pic: self.profile_pic, name: self.name, bio: self.bio, password_hash: self.password_hash, wg: self.wg, username:self.username,
            revoke_before: self.revoke_before.assume_utc() 
        }
    }
}


pub struct TryIdentity(Option<Identity>);
pub struct MaybeIdentity(Result<Identity, actix_web::Error>);

#[derive(ThisErrorError, Debug)]
enum AuthError {
    #[error("Database-Connector ran into an Error")]
    Database( #[from] sqlx::error::Error ),
    #[error("Concurrency Error :(")]
    Blocking(#[from] actix_web::error::BlockingError),
    #[error("JWT Token malformed, forged, outdated, not applicable or otherwise unusable. Shame.")]
    JWTForged( #[from] jsonwebtoken::errors::Error ),
    #[error("The JWT was revoked, or was -paradoxically- issued before this Object even existed")]
    JWTRevoked,
    #[error("The Object Referenced in the JWToken does not exist. (e.g User has been deleted, etc)")]
    ObjectGone
}
impl actix_web::error::ResponseError for AuthError {
    fn status_code(&self) -> StatusCode {
        match *self {
            AuthError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AuthError::Blocking(_) => StatusCode::INTERNAL_SERVER_ERROR,
            _ => StatusCode::FORBIDDEN
        }
    }
}

#[derive(Deserialize)]
struct LoginInfo {
    username: String,
    password: String
}

#[derive(Deserialize)]
pub struct Pwd {
    password: String
}

#[derive(ThisErrorError, Debug)]
enum LoginError {
    #[error("Failed to hash and compare Password")]
    Hashing( #[from] password_hash::errors::Error ),
    #[error("User has an invalid PasswordHash in the Database -> can't login as them, contact support")]
    HashParsing( password_hash::errors::Error ),
    #[error("Database-Connector ran into an Error")]
    Database( #[from] sqlx::error::Error ),
    #[error("JWT Token could't be issued")]
    JWT( #[from] jsonwebtoken::errors::Error ),
    #[error("Concurrency Error :(")]
    Blocking(#[from] actix_web::error::BlockingError),
    #[error("Wrong login_info, hurensohn!")]
    WrongCredentials
}
impl actix_web::error::ResponseError for LoginError {
    fn status_code(&self) -> StatusCode {
        match *self {
            LoginError::WrongCredentials => StatusCode::FORBIDDEN,
            LoginError::HashParsing(_) => StatusCode::CONFLICT,
            _ => StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

// ================================================================================== ROUTES ==================================================================================
#[get("/priviledged")]
async fn priviledged(mut auth : Identity) -> impl Responder {
    trace!("User sucessfully authenticated");

    auth.password_hash = "<Not Provided>".to_string();

    HttpResponse::Ok()
        .append_header(( "CONTENT-TYPE", "text/html; charset=UTF-8" ))
        .body(format!(
            r#"<body>Greetings Aristocrat!<br>
            <img height="300" src="/public/img/greetings_aristocrat.jpg"/><br>
            You successfully authenticated!!<br>
            {:#?}</body>"#, auth)
        )
}


#[get("/classist")]
async fn classist(auth: MaybeIdentity) -> impl Responder {
    match auth.0 {
        Ok(identity) => {
            HttpResponse::Ok()
                .content_type(ContentType::html())
                .body(format!(r#"
                <body>
                    <h1>Ahhhh Lord {}</h1>
                    <p>Such a pleasure to meet again<br>
                        <sub>(You appear to be fitted the fuck out, head to toe, in some damn JWTokens🥶🥵)</sub>
                    </p>
                    <img height="300" src="/public/img/recognition.jpg"/>
                "#, identity.name))
        }
        Err(_) => {
            HttpResponse::Ok()
                .content_type(ContentType::html())
                .body(format!(r#"
                <body>
                    <h1>Lmao nah yain't gonna pull up like this</h1>
                    <p>Plebs like you aren't welcome here<br>
                        <sub>(You pulled up without any drip 🥱😑 (No JWToken=No Bitches))</sub>
                    </p>
                    <img height="300" src="/public/img/rejection.jpg"/>
                "#))
        }
    }
}

#[post("/login")]
async fn login_handler(conn: ConnectionInfo, login_info: web::Json<LoginInfo>) -> Result<impl Responder, Error> {
    trace!("Requestee {:?} attempting to log in with: {}", conn.realip_remote_addr(), login_info.username); // trust realip

    let jwt = login(login_info.into_inner()).await?;

    Ok( HttpResponse::Ok().json(json!(
        {
            "token": jwt,
            "expires": get_current_timestamp() + 31557600
        }
    ))) 
}

#[get("/login_unsafe")]
async fn unsafe_login_handler(conn: ConnectionInfo, login_info: web::Query<LoginInfo>) -> Result<impl Responder, Error> {
    trace!("Requestee {:?} attempting to log in with: {}", conn.realip_remote_addr(), login_info.username); // trust realip

    let jwt = login(login_info.into_inner()).await?;

    Ok( HttpResponse::Ok()
        .cookie(
            Cookie::build(&cookie_name("auth_token"), &jwt).http_only(true).permanent().path("/").finish()
        ).json(json!(
            {
                "token": jwt,
                "expires": get_current_timestamp() + 31557600
            }
        ))
    ) 
}

#[get("/hash_password_unsafely_use_only_for_tests")]
pub async fn hash_password(lmao: web::Query<Pwd>) -> Result<impl Responder, Error> {
    let password = lmao.password.as_bytes(); // Bad password; don't actually use!
    let salt = SaltString::generate(&mut OsRng);

    // Hash password to PHC string ($pbkdf2-sha256$...)
    let password_hash = Pbkdf2.hash_password(password, &salt)
        .map_err(|e | res_error(StatusCode::CONFLICT, Some(e), "Failed to has"))?
        .serialize();
    
    Ok(
        HttpResponse::Ok()
        .body(password_hash.to_string())
    )
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            //.wrap(Authentication)
            .service(priviledged)
            .service(classist)
            .service(login_handler)
            .service(unsafe_login_handler)
            .service(hash_password)
    );
}


// ================================================================================== AUTH Logic ==================================================================================
// Login 
async fn login ( login_info: LoginInfo ) -> Result<String, LoginError> {
    // Match a user
    let user = sqlx::query_as!(Identity, "SELECT * FROM users WHERE username = $1;", login_info.username)
        .fetch_one(DB_POOL.get().await).await
        .map_err( |db_err| {
            match db_err {
                sqlx::Error::RowNotFound => LoginError::WrongCredentials,
                _ => LoginError::Database(db_err),
            }
        })?;

    // hashing and jwt encodeing can take some time
    web::block( move || -> Result<String, LoginError> {
        // Check if user is correct
        let parsed_hash = PasswordHash::new(&user.password_hash.trim()).map_err(|e| LoginError::HashParsing(e))?;

        if Pbkdf2.verify_password(login_info.password.as_bytes(), &parsed_hash).is_ok() {
            let jwt =encode(&Header::new(JWT_ALGO), &JWTClaims {
                auth_as: user.id,
                aud: "USER".to_string(),
                iss: JWT_ISS.to_string(),
                iat: get_current_timestamp(),
                exp: get_current_timestamp() + 31557600
            }, &JWT_ENC_KEY)?;
            Ok(jwt)
        } else {
            Err( LoginError::WrongCredentials )
        }
    }).await?
}

async fn authenticate(provided_token : &str) -> Result<TryIdentity, AuthError> {
    // Block on JWT Decode
    let provided_token = provided_token.to_owned();
    let token_d = 
        web::block( move || -> Result<TokenData<JWTClaims>, AuthError> {
            Ok(decode::<JWTClaims>(&provided_token, &JWT_DEC_KEY, &JWT_VAL)?)
        }).await??;
    
    
    let user = sqlx::query_as!(Identity, "SELECT * FROM users WHERE id = $1;", token_d.claims.auth_as)
        .fetch_one( DB_POOL.get().await).await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AuthError::ObjectGone,
            _ => AuthError::Database(e),
        })?;

    let iat_time = OffsetDateTime::from_unix_timestamp(token_d.claims.iat.try_into().unwrap_or(i64::MAX)).unwrap_or(PrimitiveDateTime::MAX.assume_utc());
    let rev_time = user.revoke_before.assume_utc();
    if iat_time <= rev_time {
        Err(AuthError::JWTRevoked)
    } else {
        Ok(TryIdentity( Some( user ) ))
    }
}

// Route Authentication
impl FromRequest for TryIdentity {
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        Self::extract(req)
    }

    fn extract(req: &HttpRequest) -> Self::Future { 
        let req = req.clone();

        async move {
            match req.cookie(&cookie_name("auth_token")) {
                Some(auth_cookie) => Ok( authenticate(auth_cookie.value()).await? ),
                None => match req.headers().get(header::AUTHORIZATION) {
                    Some(auth_header) => {
                        let auth_header = auth_header.to_str().map_err(|e |auth_error(Some(e), "Failed to parse Auth-Header!"))?;
                        if let Some( provided_token ) = auth_header.strip_prefix("Bearer ") {
                            Ok( authenticate(provided_token).await? )
                        } else {
                            Err( auth_error::<String>(None, "Support only for BEARER Authentication! Make a request to /auth/login or /auth/register to obtain token!") )
                        }
                    },
                    None => Ok( TryIdentity(None) ),
                }
            }
        }.boxed_local()
    }
}

impl FromRequest for Identity {
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        Self::extract(req)
    }

    fn extract(req: &HttpRequest) -> Self::Future { 
        let extractlmao = TryIdentity::extract(req);
        async move {
            let maybe_identity = extractlmao.await;

            match maybe_identity {
                Ok(o) => {
                    match o.0 {
                        Some(s) => {
                            Ok(s)
                        },
                        None => Err( auth_error::<String>(None, "This Endpoint REQUIRES Bearer Authentication") )
                    }
                },
                Err(e) => Err(e),
            }
        }.boxed_local()
    }
}

impl FromRequest for MaybeIdentity {
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        Self::extract(req)
    }

    fn extract(req: &HttpRequest) -> Self::Future { 
        let extractlmao = Identity::extract(req);
        async move {
            let maybe_identity = extractlmao.await;
            Ok(MaybeIdentity(maybe_identity))
        }.boxed_local()
    }
}

// ================================================================================== DISGUSTING Helpers ==================================================================================
fn auth_error<T : std::fmt::Debug + std::fmt::Display + 'static >(source_err: Option<T>, msg: &'static str) -> Error {
    res_error(StatusCode::UNAUTHORIZED, source_err, msg)
}

pub fn res_error<T : std::fmt::Debug + std::fmt::Display + 'static >(status : StatusCode, source_err: Option<T>, msg: &str) -> Error {
    let mut res = HttpResponse::build(status);
        
    if status == StatusCode::UNAUTHORIZED { 
        res.append_header(( "WWW-Authenticate", r#"Bearer realm="WG-API DEFAULT-ROLE""#) );
    }
    
    let res = res.append_header(( "CONTENT-TYPE", "text/plain; charset=UTF-8" ))
        .body(msg.to_owned());

    match source_err {
        Some(s) => InternalError::from_response(s,res).into(),
        None => InternalError::from_response(String::from("Misc Auth Error"),res).into(),
    }
}

fn cookie_name( name: &str ) -> String {
    format!("{}_v{}_{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"), name)
}   