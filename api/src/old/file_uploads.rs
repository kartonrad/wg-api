use actix_files::NamedFile;
use actix_multipart::Multipart;
//use serde::{Serialize, Deserialize};
use thiserror::Error as ThisErrorError;
use array_macro::array;
use futures_util::{StreamExt};
use crate::{DB_POOL, db, auth::{res_error, TryIdentity}};


use actix_web::{http::StatusCode, get};
use sqlx::{Transaction, Postgres, Executor};
use tokio::{fs::{OpenOptions,self}, io::AsyncWriteExt};
use std::fs::remove_file as remove_file_sync;
use std::{
    sync::atomic::{AtomicUsize, Ordering}, str::Utf8Error
};

// ================================================================================== CONSTS/STATICS ==================================================================================

static TEMP_UPLOAD_COUNTER: AtomicUsize = AtomicUsize::new(0);

// ================================================================================== ROUTES ==================================================================================

#[get("/uploads/{name}")]
pub async fn get_uploads_service( try_identity: TryIdentity, path: actix_web::web::Path<(String,)> ) -> Result<NamedFile, actix_web::Error> {

    let filename = path.0.split(".").next().ok_or(res_error::<String>(StatusCode::UNPROCESSABLE_ENTITY, None, "??? Bozo enter real filename"))?;
    let filename_int = filename.parse::<i32>().map_err(|e| res_error(StatusCode::UNPROCESSABLE_ENTITY, Some(e), "??? Bozo filename needs to be number and extension"))?;

    let access_by_wg: Option<i32> = sqlx::query_scalar!("SELECT access_only_by_wg FROM uploads WHERE id=$1;", filename_int)
        .fetch_one(db!()).await.map_err(|e| res_error(StatusCode::INTERNAL_SERVER_ERROR, Some(e), "??? Database glitched/ didn't find your shit"))?;
    
    if let Some(req_wg) = access_by_wg {
        let error = res_error::<String>(StatusCode::FORBIDDEN, None, "YOu need to be in a specific WG to view this upload");
        if let Some(identity) = try_identity.0 {
            if let Some(wg_id) = identity.wg {
                if wg_id != req_wg {
                    return Err(error)
                }
            } else {
                return Err(error)
            }
        } else {
            return Err(error)
        }
    }

    Ok(NamedFile::open( format!("uploads/{}", path.0) )?)
}

// ================================================================================== STATE MODEL ==================================================================================

#[derive(ThisErrorError, Debug)]
pub enum UploadError {
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

use common::Upload;

pub struct AscendingUpload {
    pub temp_upload: TempUpload,
    pub global_id: i32,
    pub transaction: Transaction<'static, Postgres>
}
impl AscendingUpload {
    pub async fn ascend(mut self) -> Result<Upload, sqlx::Error>  {
        let temp_path = self.temp_upload.path();
        fs::rename(temp_path, format!("uploads/{}.{}", self.global_id, self.temp_upload.extension)).await?;
        self.transaction.commit().await?;
        self.temp_upload.cleaned = true;
        Ok(Upload {
            id: self.global_id, 
            extension: self.temp_upload.extension.to_owned(), 
            size_kb: (self.temp_upload.size/1000) as i32, 
            original_filename: self.temp_upload.original_filename.to_owned()
        })
    }
}

#[derive(Debug, Clone)]
pub struct TempUpload{
    pub local_id: usize,
    pub extension: String,
    pub original_filename: String,
    pub size: usize,
    pub cleaned: bool
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
    pub fn move_responsibility (&mut self) -> Self {
        let c = self.clone();
        self.cleaned = true;
        return c;
    }

    pub fn path(&self) -> String { 
        format!("uploads/temp/{}.{}", self.local_id, self.extension)
    }

    pub async fn clean(&mut self) {
        if self.cleaned { return }
        
        let temp_path=self.path();

        // Not neccessary to remove file, as it is temporary
        fs::remove_file(&temp_path).await.unwrap_or_else(|e| 
            warn!("Failed to delete TempUpload file, after error, at: {} -- It should be cleaned up on restart tho.\nReason: {:?}", temp_path, e)
        );
        self.cleaned = true; // dont try again in destructor, even if it failed
    }

    pub async fn into_db(self, scope_to_wg: Option<i32>) -> Result<AscendingUpload, sqlx::Error> {
        let db = db!();
        let mut transaction = db.begin().await?;

        // IGNORE long filenames, we don't need them
        let mut original_filename = self.original_filename.clone();
        if self.original_filename.len() > 200 {
            original_filename = format!("name_to_long_lmao.{}", self.extension);
        }

        let global_id: i32 = match scope_to_wg {
            Some(wg_id) => {
                sqlx::query_scalar!("INSERT INTO uploads (extension, original_filename, size_kb, access_only_by_wg) VALUES ($1, $2, $3, $4) RETURNING id;", 
                    self.extension, original_filename, (self.size/1000) as i32, wg_id)
            },
            None => sqlx::query_scalar!("INSERT INTO uploads (extension, original_filename, size_kb) VALUES ($1, $2, $3) RETURNING id;", 
                self.extension, original_filename, (self.size/1000) as i32,)
        }.fetch_one(&mut transaction).await?;

        Ok(AscendingUpload { 
            temp_upload: self, 
            global_id,
            transaction,
        })
    }

    pub async fn from_multipart_field(field: actix_multipart::Field, max_size: usize) -> Result<Self, UploadError> {
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

pub async fn multipart_parse<const NR_STRS: usize, const NR_FILES: usize>(mut payload: Multipart, string_fields: [&str;NR_STRS], file_fields: [&str;NR_FILES]) 
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

pub async fn delete_unreferenced_upload<'a,T>(formerup: i32, transaction: T) -> Result<(), sqlx::Error>
where T: Executor<'a, Database=Postgres>
{
    let row = sqlx::query!( "DELETE FROM uploads WHERE id = $1 RETURNING extension, original_filename;", formerup)
        .fetch_one(transaction).await?;
    
    let del_path = format!("uploads/{}.{}", formerup, row.extension);
    match fs::remove_file( &del_path ).await {
        Ok(_) => info!("Successfully removed old upload {}, (original filename: {:?})", del_path, row.original_filename),
        Err(e) => warn!("Failed to delete permanent Upload file, after error, at: {} -- It has now become orphaned!!\nReason: {:?}", del_path, e)
    }
    Ok(())
}


/**
 * BEHOLD!
 * The most cursed Macro in existence!!
 */
#[macro_export]
macro_rules! change_upload {
    ($x:literal, $y:literal, $t:ty) => { 
        {
        use futures_util::future::LocalBoxFuture;
        use futures_util::FutureExt;
        use std::concat;
        use sqlx::postgres::PgArguments;
        use common::Upload;
        use crate::file_uploads::delete_unreferenced_upload;
        use sqlx::Arguments;
        |temp: TempUpload, row_id: $t, scope_to_wg: Option<i32>| -> LocalBoxFuture<'static, Result<Upload, sqlx::Error>> {
            let row_id = row_id.to_owned();
            async move {
                let mut ascending = temp.into_db(scope_to_wg).await?;

                let mut select_args = PgArguments::default();
                select_args.add(row_id);

                let (former_upload,) : (Option<i32>,) = sqlx::query_as_with(concat!("SELECT ", $y, " FROM ", $x, " WHERE id = $1;"), select_args)
                    .fetch_one(&mut ascending.transaction).await?;

                let mut update_args = PgArguments::default();
                update_args.add(ascending.global_id);
                update_args.add(row_id);

                sqlx::query_with(concat!("UPDATE ", $x, " SET ", $y ," = $1 WHERE id = $2;"), update_args)
                    .execute(&mut ascending.transaction).await?;

                if let Some(formerup) = former_upload {
                    delete_unreferenced_upload(formerup, &mut ascending.transaction).await?;
                }

                Ok(ascending.ascend().await?)
            }.boxed_local()
        }
        }
    };
}

/* 
pub async fn replace_upload(temp: TempUpload)  {
    let mut ascending = temp.into_db().await.unwrap();
    sqlx::query!("UPDATE users SET profile_pic = $1 WHERE id = $2", ascending.global_id, identity.id)
        .execute(&mut ascending.transaction).await.unwrap();
    ascending.ascend().await.unwrap();
}*/


/*
let mut ascending = profile_picf.into_db().await.unwrap();
        sqlx::query!("UPDATE users SET profile_pic = $1 WHERE id = $2", ascending.global_id, identity.id)
            .execute(&mut ascending.transaction).await.unwrap();
        ascending.ascend().await.unwrap(); */