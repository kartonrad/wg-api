use actix_multipart::Multipart;
use actix_web::{ HttpResponse, Responder, get, put, delete, post, http::StatusCode, dev::{ConnectionInfo}, web, Error,};
use futures_util::StreamExt;
use serde::Serialize;
use sqlx::{Transaction, Postgres};
use crate::DB_POOL;

use super::auth::Identity;
use tokio::{fs::{OpenOptions,self}, io::AsyncWriteExt};
use std::fs::remove_file as remove_file_sync;
use std::{
    sync::atomic::{AtomicUsize, Ordering}, str::Utf8Error
};
use thiserror::Error as ThisErrorError;
use array_macro::array;

// ================================================================================== CONSTS/STATICS ==================================================================================

static TEMP_UPLOAD_COUNTER: AtomicUsize = AtomicUsize::new(0);

// ================================================================================== STATE MODEL ==================================================================================

#[derive(ThisErrorError, Debug)]
enum UploadError {
    #[error("File Size can't exceed {}MB!", .0/1000000)]
    MaxSizeExceeded(usize),
    #[error("The multipart/form-data header appears to be missing Information on (whats supposed to be) the File (check filename/extension header)")]
    NotFile,
    #[error("Multipart Form failed.")]
    Multipart( #[from] actix_multipart::MultipartError ),
    #[error("Couldn't create/delete/update file.")]
    IO( #[from] std::io::Error ),
    #[error("Nahhh bro you tripping. The Field '{0}' has nothing to do with this endpoint, gett it outta here 🙄")]
    UnknownField( String ),
    #[error("💀💀 Why is field '{0}' empty??? Why send it in the first place????")]
    EmptyTextField( String ),
    #[error("Bozo sent a text field with invalid UTF8 🤡🤡")]
    InvalidUTF8TextField( #[from] Utf8Error )
}
impl actix_web::error::ResponseError for UploadError {
    fn status_code(&self) -> StatusCode {
        match *self {
            UploadError::MaxSizeExceeded(_) => StatusCode::PAYLOAD_TOO_LARGE,
            UploadError::NotFile => StatusCode::BAD_REQUEST,
            UploadError::UnknownField(_) => StatusCode::BAD_REQUEST,
            UploadError::EmptyTextField(_) => StatusCode::BAD_REQUEST,
            UploadError::InvalidUTF8TextField(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

struct AscendingUpload {
    temp_upload: TempUpload,
    global_id: i32,
    transaction: Transaction<'static, Postgres>
}
impl AscendingUpload {
    async fn ascend(mut self) -> Result<(), sqlx::Error> {
        let temp_path = self.temp_upload.path();
        let global_path = fs::rename(temp_path, format!("uploads/{}.{}", self.global_id, self.temp_upload.extension)).await?;
        self.transaction.commit().await?;
        self.temp_upload.cleaned = true;
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct TempUpload{
    local_id: usize,
    extension: String,
    original_filename: String,
    size: usize,
    cleaned: bool
}

impl Drop for TempUpload {
    fn drop(&mut self) {
        if self.cleaned { return }
        
        let temp_path=format!("uploads/temp/{}.{}", self.local_id, self.extension);
        warn!("Synchronously removing '{}'!!! (Synch IO harms concurrency!!!)", temp_path);

        // Not neccessary to remove file, as it is temporary
        remove_file_sync(&temp_path).unwrap_or_else(|e| 
            warn!("Failed to delete TempUpload file, after error, at: {} -- It should be cleaned up on restart tho.\nReason: {:?}", temp_path, e)
        );
    }
}

impl TempUpload {
    fn move_responsibility (&mut self) -> Self {
        let c = self.clone();
        self.cleaned = true;
        return c;
    }

    fn path(&self) -> String { 
        format!("uploads/temp/{}.{}", self.local_id, self.extension)
    }

    async fn clean(&mut self) {
        if self.cleaned { return }
        
        let temp_path=self.path();

        // Not neccessary to remove file, as it is temporary
        fs::remove_file(&temp_path).await.unwrap_or_else(|e| 
            warn!("Failed to delete TempUpload file, after error, at: {} -- It should be cleaned up on restart tho.\nReason: {:?}", temp_path, e)
        );
        self.cleaned = true; // dont try again in destructor, even if it failed
    }

    async fn into_db(self) -> Result<AscendingUpload, sqlx::Error> {
        let db = DB_POOL.get().await;
        let mut transaction = db.begin().await?;

        let global_id = sqlx::query_scalar!("INSERT INTO uploads (extension, original_filename, size_kb) VALUES ($1, $2, $3) RETURNING id;", 
            self.extension, self.original_filename, (self.size/1000) as i32
        ).fetch_one(&mut transaction).await?;

        Ok(AscendingUpload { 
            temp_upload: self, 
            global_id,
            transaction,
        })
    }

    async fn from_multipart_field(field: actix_multipart::Field, max_size: usize) -> Result<Self, UploadError> {
        let cd = field.content_disposition();

        let local_id = TEMP_UPLOAD_COUNTER.fetch_add(1, Ordering::SeqCst);
        let original_filename = cd.get_filename().ok_or(UploadError::NotFile)?.to_owned();
        let extension = original_filename.split(".").last().unwrap_or("").to_owned();

        let temp_path=format!("uploads/temp/{}.{}", local_id, extension);
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(temp_path.to_owned()).await?;

        // write remaining shit to file
        let size: usize = match write_field_to_file(field, &mut file, max_size).await {
            Ok(o) => o,
            Err(e) => {
                let _ = file.shutdown().await;
                drop(file); // did the shutdown fail? too bad! we're gonna get rid of the file handle anyway (shouldn't matter too much honestly)

                // Not neccessary to remove file, as it is temporary
                fs::remove_file(&temp_path).await.unwrap_or_else(|e| 
                    warn!("Failed to delete TempUpload file, after error, at: {} -- It should be cleaned up on restart tho.\nReason: {:?}", temp_path, e)
                );
                return Err(e);
            }
        };

        Ok(TempUpload {
            local_id,
            extension, 
            original_filename, 
            size,
            cleaned: false
        })
    }
}

async fn write_field_to_file(mut field: actix_multipart::Field, file: &mut tokio::fs::File, max_size: usize) -> Result<usize, UploadError> {
    let mut size: usize = 0;
    while let Some(chunk) = field.next().await {
        let chunk = chunk?;
        size += chunk.len();
        if size > max_size /* 40mb */ {
            return Err( UploadError::MaxSizeExceeded(max_size) );
        }

        file.write_all(&chunk).await?;
    }
    file.flush().await?;

    Ok(size)
}

// ================================================================================== ROUTES ==================================================================================
#[get("/me")]
async fn get_user_me(mut identity: Identity) -> impl Responder {
    identity.password_hash = "<Not Provided>".to_string();

    HttpResponse::Ok()
        .json(identity)
}


async fn multipart_parse<const NR_STRS: usize, const NR_FILES: usize>(mut payload: Multipart, string_fields: [&str;NR_STRS], file_fields: [&str;NR_FILES]) 
    -> Result<( [Option<String>;NR_STRS], [Option<TempUpload>;NR_FILES] ) , UploadError> 
{
    let mut strings: [Option<String>;NR_STRS] = array![None; NR_STRS];
    let mut files: [Option<TempUpload>;NR_FILES] = array![None; NR_FILES];

    while let Some(item) = payload.next().await {
        let mut field = item?;
        //trace!("Field: {:?}", field);

        if let Some(pos) = string_fields.iter().position(|&s| s == field.name()) {
            if let Some(chunk) = field.next().await {
                let cchunk = &chunk?;
                let string = std::str::from_utf8(cchunk)?;
                strings[pos] = Some(string.to_string());
            } else {
                return Err( UploadError::EmptyTextField(field.name().to_string()) )
            }
        } else if let Some(pos) = file_fields.iter().position(|&s| s == field.name()) {
            let cd = field.content_disposition();
            let mut valid_file = false;
            if cd.is_form_data() {
                if let Some(_) = cd.get_filename() {
                    valid_file = true;
                    let temp_upload = TempUpload::from_multipart_field(field, 40000000).await?;
                    files[pos] = Some(temp_upload);
                }
            }
            if !valid_file {
                return Err( UploadError::NotFile );
            }
        } else {
            return Err( UploadError::UnknownField(field.name().to_string()) );
        }
    }

    Ok(( strings, files ))
}


#[put("/me")]
async fn put_user_me(identity: Identity, mut payload: Multipart) -> Result<&'static str, Error> {
    // iterate over multipart stream
    #[derive(Serialize)]
    struct ResJson {
        name: bool,
        bio: bool,
        username: bool,
        profile_pic: bool
    }

    let mut lmaobozo = multipart_parse(payload, ["name", "bio", "username"], ["profile_pic"]).await?;
    trace!("Bozo fields: {:?}", lmaobozo);

    if let Some(profile_picf) = &mut lmaobozo.1[0] {
        let profile_picf = profile_picf.move_responsibility();
        let mut ascending = profile_picf.into_db().await.unwrap();
        let rows_affected = sqlx::query!("UPDATE users SET profile_pic = $1 WHERE id = $2", ascending.global_id, identity.id)
            .execute(&mut ascending.transaction).await.unwrap();
        ascending.ascend().await.unwrap();
    }
    
    Ok("Success!!")
}

// user_change_password, user_revoke_tokens

#[get("/my_wg")]
async fn get_wg(identity: Identity) -> impl Responder {
    todo!();
    ""
}

#[put("/my_wg")]
async fn put_wg(identity: Identity) -> impl Responder {
    todo!();
    ""
}

#[get("/my_wg/costs")]
async fn get_wg_costs(identity: Identity) -> impl Responder {
    todo!();
    ""
}

#[post("/my_wg/costs")]
async fn post_wg_costs(identity: Identity) -> impl Responder {
    todo!();
    ""
}

#[get("/my_wg/costs/{id}")]
async fn get_wg_costs_id(identity: Identity) -> impl Responder {
    todo!();
    ""
}

#[put("/my_wg/costs/{id}")]
async fn put_wg_costs_id(identity: Identity) -> impl Responder {
    todo!();
    ""
}

#[delete("/my_wg/costs/{id}")]
async fn delete_wg_costs_id(identity: Identity) -> impl Responder {
    todo!();
    ""
}

#[get("/my_wg/costs/stats")]
async fn get_wg_costs_stats(identity: Identity) -> impl Responder {
    todo!();
    ""
}


pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            //.wrap(Authentication)
            .service(get_user_me)
            .service(put_user_me)
            .service(get_wg)
            .service(put_wg)
            .service(get_wg_costs)
            .service(post_wg_costs)
            .service(get_wg_costs_id)
            .service(put_wg_costs_id)
            .service(delete_wg_costs_id)
            .service(get_wg_costs_stats)

    );
}

/*
 let filepath=format!("uploads/temp/{}{}", temp_upload.local_id, match get_mime_extensions(field.content_type()) {
            Some(ext) => {
                let mut str = ".".to_string();
                str.push_str( ext.first().map(|s| *s).unwrap_or("cringe") );
                str
            },
            None=>"".to_string()
        } );
 */