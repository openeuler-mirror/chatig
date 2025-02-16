use actix_web::{get, post, web, delete, Error, HttpResponse, Responder};
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use tokio_postgres::NoTls;
use serde::Deserialize;
use serde::Serialize;
use actix_web::error::ErrorInternalServerError;
use rand::{thread_rng, Rng};
use std::iter;
use serde_yaml::to_string;
use chrono::Utc;
use tokio_postgres::Client;

use crate::meta::init::get_pool;

// use crate::servers::api_schemas::{AppState, InvitationCodeRequest, InvitationCodeResponse};
use crate::apis::control_api::schemas::{InvitationCodeRequest, InvitationCodeResponse};

#[allow(dead_code)]
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_all_invitation_codes)
       .service(get_invitation_codes_by_user)
       .service(delete_invitation_code_by_id)
       .service(allocate_invitation_code_to_user)
       .service(change_invitation_code_database_size);
}

// Invitation code database
#[derive(Serialize, Deserialize, Debug)]
pub struct InvitationCode {
    pub id: i32,           
    pub user: String,
    pub created_at: i64,
    pub origination: String,
    pub telephone: String,
    pub email: String,
    pub code: String,
}

// Function to generate a specified number of invitation codes and save them into the database.
pub async fn generate_and_save_invitation_codes(pool: &Pool<PostgresConnectionManager<NoTls>>) -> Result<impl Responder, Error> {

    // Query statement to check if the table exists.
    // Get the database connection object, which will be used to check if the table exists and for possible subsequent table creation operations.
    let client = match pool.get().await {
        Ok(client) => client,
        Err(err) => {
            return Err(ErrorInternalServerError(format!("Failed to get database connection: {}", err)));
        }
    };

    // Query statement to count the number of data records in the table.
    let count_query = "SELECT COUNT(*) FROM invitation_code";
    let count_result = client
       .query_one(count_query, &[])
       .await
       .map_err(|err| ErrorInternalServerError(format!("Failed to count records in table: {}", err)))?;
    let record_count: i64 = count_result.get(0);

    if record_count > 0 {
        return Ok(HttpResponse::Ok().body("invitation_code table already has data, skipping code generation."));
    }

    let mut invitation_codes = Vec::new();

    // Generate invitation codes in a loop.
    for _ in 0..10 {
        // Generate default values for other fields related to the invitation code (in this example, some use empty strings, and 'created_at' uses the current timestamp, which can be adjusted according to the actual business).
        let user = "".to_string();
        let origination = "".to_string();
        let telephone = "".to_string();
        let email = "".to_string();
        let created_at = Utc::now().timestamp() as i64;

        // Get the database connection object.
        let client = match pool.get().await {
            Ok(client) => client,
            Err(err) => {
                return Err(ErrorInternalServerError(format!("Failed to get database connection: {}", err)));
            }
        };

        // Generate invitation code. Here, call the function with uniqueness check.
        let sk_code = generate_sk_code(&client).await?;

        // Construct a complete InvitationCode struct instance.
        let invitation_code = InvitationCode {
            id: 0,
            user,
            created_at,
            origination,
            telephone,
            email,
            code: sk_code,
        };

        invitation_codes.push(invitation_code);
    }

    // Batch insert invitation codes into the database.
    let client = match pool.get().await {
        Ok(client) => client,
        Err(err) => {
            return Err(ErrorInternalServerError(format!("Failed to get database connection: {}", err)));
        }
    };

    let query = "
        INSERT INTO invitation_code (users, origination, created_at, telephone, email, code)
        VALUES ($1, $2, $3, $4, $5, $6)";

    for invitation_code in invitation_codes {
        match client.execute(query, &[
            &invitation_code.user,
            &invitation_code.origination,
            &invitation_code.created_at,
            &invitation_code.telephone,
            &invitation_code.email,
            &invitation_code.code,
        ]).await {
            Ok(_) => {}
            Err(err) => {
                return Err(ErrorInternalServerError(format!("Failed to insert invitation code info: {}", err)));
            }
        }
    }

    Ok(HttpResponse::Ok().body("50 invitation codes have been successfully generated and saved to the database."))
}

// Generate 32-character random invitation codes and ensure uniqueness.
pub async fn generate_sk_code(client: &Client) -> Result<String, Error> {
    loop {
        let prefix = "sk-";
        let characters: Vec<char> = iter::repeat(())
         .take(29)
         .map(|_| {
                let num = thread_rng().gen_range(0..62);
                match num {
                    n if n < 10 => (n as u8 + b'0') as char,
                    n if n < 36 => (n as u8 - 10 + b'A') as char,
                    _ => (num as u8 - 36 + b'a') as char,
                }
            })
         .collect();
        let code = format!("{}{}", prefix, characters.into_iter().collect::<String>());

        // Query the database to check if the invitation code already exists.
        let check_query = "SELECT COUNT(*) FROM invitation_code WHERE code = $1";
        let count = client
          .query_one(check_query, &[&code])
          .await
          .map_err(|err| ErrorInternalServerError(format!("Failed to check code uniqueness: {}", err)))?;
        let count: i64 = count.get(0);
        if count == 0 {
            return Ok(code);
        }
    }
}

// Get all invitation codes
#[get("invitation")]
pub async fn get_all_invitation_codes() -> Result<impl Responder, Error> {
    // Get a connection from the database connection pool.
    let pool = get_pool().await?;
    let client = match pool.get().await {
        Ok(client) => client,
        Err(err) => {
            return Err(ErrorInternalServerError(format!("Failed to get database connection: {}", err)));
        }
    };

    // Construct a query statement to select all columns in the invitation_code table.
    let query = "SELECT * FROM invitation_code";
    let rows = match client.query(query, &[]).await {
        Ok(rows) => rows,
        Err(err) => {
            return Err(ErrorInternalServerError(format!("Failed to execute query: {}", err)));
        }
    };

    // A vector used to store all the invitation code records.
    let mut invitation_codes: Vec<InvitationCode> = Vec::new();

    // Traverse each row of the query result, convert the data into instances of the InvitationCode struct, and store them in the vector.
    for row in rows {
        let id: i32 = row.get("id");
        let user: String = row.get("users");
        let created_at: i64 = row.get("created_at");
        let origination: String = row.get("origination");
        let telephone: String = row.get("telephone");
        let email: String = row.get("email");
        let code: String = row.get("code");

        let invitation_code = InvitationCode {
            id,
            user,
            created_at,
            origination,
            telephone,
            email,
            code,
        };

        invitation_codes.push(invitation_code);
    }

    // Serialize the vector of invitation code records into a JSON string.
    let json_response = serde_json::to_string(&invitation_codes).map_err(|err| {
        ErrorInternalServerError(format!("Failed to serialize response: {}", err))
    })?;

    // Return an HttpResponse with JSON-formatted data.
    Ok(HttpResponse::Ok()
       .content_type("application/json")
       .body(json_response))
}

// Get detail invitation code info by user
#[get("invitation/user")]
pub async fn get_invitation_codes_by_user(
    req_body: web::Json<InvitationCodeRequest>, 
) -> Result<impl Responder, Error> {

    let pool = get_pool().await?;
    let client = match pool.get().await {
        Ok(client) => client,
        Err(err) => {
            return Err(ErrorInternalServerError(format!("Failed to get database connection: {}", err)));
        }
    };

    let username= req_body.user.to_string();

    // Build a query statement to find the corresponding invitation code records according to the username.
    let query = "SELECT * FROM invitation_code WHERE users = $1";
    let rows = match client.query(query, &[&username]).await {
        Ok(rows) => rows,
        Err(err) => {
            return Err(ErrorInternalServerError(format!("Failed to execute query: {}", err)));
        }
    };

    // A vector used to store the invitation code records of this user.
    let mut invitation_codes: Vec<InvitationCode> = Vec::new();

    // Traverse each row of the query result, convert the data into instances of the InvitationCode struct and store them in the vector.
    for row in rows {
        let id: i32 = row.get("id");
        let user: String = row.get("users");
        let created_at: i64 = row.get("created_at");
        let origination: String = row.get("origination");
        let telephone: String = row.get("telephone");
        let email: String = row.get("email");
        let code: String = row.get("code");

        let invitation_code = InvitationCode {
            id,
            user,
            created_at,
            origination,
            telephone,
            email,
            code,
        };

        invitation_codes.push(invitation_code);
    }

    // Serialize the vector of invitation code records into a JSON string.
    let json_response = serde_json::to_string(&invitation_codes).map_err(|err| {
        ErrorInternalServerError(format!("Failed to serialize response: {}", err))
    })?;

    // Return an HttpResponse with JSON-formatted data and set the appropriate Content-Type to application/json.
    Ok(HttpResponse::Ok().content_type("application/json").body(json_response))
}

// Delete invitation code by id
#[delete("invitation/{id}")]
pub async fn delete_invitation_code_by_id(
    path: web::Path<i32>, // Get the id through the path parameter. Here the type is modified to i32, corresponding to the id field type in the database.
) -> Result<impl Responder, Error> {
    
    let id = path.into_inner();

    // Get a connection from the database connection pool.
    let pool = get_pool().await?;
    let client = match pool.get().await {
        Ok(client) => client,
        Err(err) => {
            return Err(ErrorInternalServerError(format!("Failed to get database connection: {}", err)));
        }
    };

    // Build a delete statement to delete the corresponding invitation code records according to the id.
    let query = "DELETE FROM invitation_code WHERE id = $1";
    match client.execute(query, &[&id]).await {
        Ok(rows_affected) => {
            if rows_affected > 0 {
                // If records are deleted, return a success prompt message.
                Ok(HttpResponse::Ok().body(format!("Deleted {} invitation code record(s) with id {}.", rows_affected, id)))
            } else {
                // If no records with the corresponding id are found, return the corresponding prompt message.
                Ok(HttpResponse::NotFound().body(format!("No invitation code record found with id {}.", id)))
            }
        },
        Err(err) => {
            // If an error occurs during the delete operation, return an internal server error response and the error message.
            Err(ErrorInternalServerError(format!("Failed to delete invitation code record with id {}: {}", id, err)))
        }
    }
}

// Allocate an invitation code to a user.
#[post("invitation")]
pub async fn allocate_invitation_code_to_user(
    req_body: web::Json<InvitationCodeRequest>, 
) -> Result<impl Responder, Error> {
        
    // Save the invitation code to PostgreSQL.
    let pool = get_pool().await?;
    let client = match pool.get().await {
        Ok(client) => client,
        Err(err) => {
            return Err(ErrorInternalServerError(format!("Failed to get database connection: {}", err)));
        }
    };

    // Find the invitation code that has not been allocated to a user.
    let find_unused_code_query = "SELECT * FROM invitation_code WHERE users = '' LIMIT 1";
    let unused_code_row = match client.query_one(find_unused_code_query, &[]).await {
        Ok(row) => row,
        Err(err) => {
            return Err(ErrorInternalServerError(format!("Failed to find unused invitation code: {}", err)));
        }
    };

    let id: i32 = unused_code_row.get("id");
    let existing_code: String = unused_code_row.get("code");

    // Construct an instance of the InvitationCode struct to be updated, using the id of the found unallocated record and update other field information.
    let invitation_code = InvitationCode {
        id,
        user: req_body.user.clone(),
        created_at: Utc::now().timestamp() as i64,
        origination: req_body.origination.clone().unwrap_or("".to_string()),
        telephone: req_body.telephone.clone().unwrap_or("".to_string()),
        email: req_body.email.clone().unwrap_or("".to_string()),
        code: existing_code,
    };
    
    // The update statement, update each field of the corresponding record according to the id, and handle the conflict situation of the unique constraint of the 'code' field (using ON CONFLICT DO UPDATE here).
    let update_query = "
        INSERT INTO invitation_code (id, users, origination, created_at, telephone, email, code)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (code)
        DO UPDATE SET
            users = EXCLUDED.users,
            origination = EXCLUDED.origination,
            created_at = EXCLUDED.created_at,
            telephone = EXCLUDED.telephone,
            email = EXCLUDED.email;";

    match client.execute(update_query, &[
        &invitation_code.id,
        &invitation_code.user,
        &invitation_code.origination,
        &invitation_code.created_at,
        &invitation_code.telephone,
        &invitation_code.email,
        &invitation_code.code,
    ]).await {
        Ok(_) => {
            // Create an instance of the response struct and fill in the invitation code information.
            let response = InvitationCodeResponse {
                id: invitation_code.code,
            };
            let json_response = to_string(&response).map_err(|err| ErrorInternalServerError(format!("Failed to serialize response: {}", err)))?;
            Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(json_response))
        },
        Err(err) => Err(ErrorInternalServerError(format!("Failed to update invitation code info: {}", err))),
    }
}

// The number of records in the database.
#[derive(Deserialize)]
struct ChangeSizeRequest {
    target_size: i64,
}

// Increase or decrease the number of records in the database.
#[post("chatig")]
pub async fn change_invitation_code_database_size(
    req_body: web::Json<ChangeSizeRequest>,
) -> Result<impl Responder, Error> {

    let target_size = req_body.target_size;

    let pool = get_pool().await?;
    let client = match pool.get().await {
        Ok(client) => client,
        Err(err) => {
            return Err(ErrorInternalServerError(format!("Failed to get database connection: {}", err)));
        }
    };

    // Query the number of invitation code records in the current database.
    let count_query = "SELECT COUNT(*) FROM invitation_code";
    let count_result = client
      .query_one(count_query, &[])
      .await
      .map_err(|err| ErrorInternalServerError(format!("Failed to count records in table: {}", err)))?;
    let mut current_count: i64 = count_result.get(0);

    if target_size < current_count {
        // Shrink operation.
        // First, query the ids of all records whose 'users' field is empty.
        let find_empty_user_ids_query = "SELECT id FROM invitation_code WHERE users = ''";
        let empty_user_ids_rows = match client.query(find_empty_user_ids_query, &[]).await {
            Ok(rows) => rows,
            Err(err) => {
                return Err(ErrorInternalServerError(format!("Failed to query empty user ids: {}", err)));
            }
        };

        // Construct a delete statement, delete records according to the queried ids, and delete one record each time until the target size is reached.
        for row in empty_user_ids_rows {
            let id: i32 = row.get("id");
            let delete_query = "DELETE FROM invitation_code WHERE id = $1";
            match client.execute(delete_query, &[&id]).await {
                Ok(_) => {
                    if current_count - 1 == target_size {
                        break;
                    }
                    current_count -= 1;
                }
                Err(err) => {
                    return Err(ErrorInternalServerError(format!("Failed to delete record during shrink operation: {}", err)));
                }
            }
        }

        // Query the current number of records in the database again to ensure it is consistent with the target size.
        let check_count_query = "SELECT COUNT(*) FROM invitation_code";
        let check_count_result = client
           .query_one(check_count_query, &[])
           .await
           .map_err(|err| ErrorInternalServerError(format!("Failed to check record count after shrink operation: {}", err)))?;
        let actual_count: i64 = check_count_result.get(0);

        if actual_count!= target_size {
            return Err(ErrorInternalServerError(format!("Shrink operation failed. Expected {} records, but found {} records in the database.", target_size, actual_count)));
        }


    } else if target_size > current_count {
        // Expand operation.
        let additional_count = target_size - current_count;
        let mut new_invitation_codes = Vec::new();
        for _ in 0..additional_count {
            // Generate default values for other fields related to the invitation code (in this example, some use empty strings, and 'created_at' uses the current timestamp, which can be adjusted according to the actual business).
            let user = "".to_string();
            let origination = "".to_string();
            let telephone = "".to_string();
            let email = "".to_string();
            let created_at = Utc::now().timestamp() as i64;

            // Generate invitation code, call the function with uniqueness check here.
            let sk_code = generate_sk_code(&client).await?;

            // Construct a complete InvitationCode struct instance.
            let invitation_code = InvitationCode {
                id: 0,
                user,
                created_at,
                origination,
                telephone,
                email,
                code: sk_code,
            };

            new_invitation_codes.push(invitation_code);
        }

        // Batch insert invitation codes into the database.
        let query = "
            INSERT INTO invitation_code (users, origination, created_at, telephone, email, code)
            VALUES ($1, $2, $3, $4, $5, $6)";

        for invitation_code in new_invitation_codes {
            match client.execute(query, &[
                &invitation_code.user,
                &invitation_code.origination,
                &invitation_code.created_at,
                &invitation_code.telephone,
                &invitation_code.email,
                &invitation_code.code,
            ]).await {
                Ok(_) => {}
                Err(err) => {
                    return Err(ErrorInternalServerError(format!("Failed to insert invitation code info during expand operation: {}", err)));
                }
            }
        }
    }

    Ok(HttpResponse::Ok().body(format!("Database size has been successfully adjusted to {}", target_size)))
}

// check if invitation code exists
pub async fn check_invitation_code_exists(
    pool: &Pool<PostgresConnectionManager<NoTls>>,
    target_code: &str,
) -> Result<bool, Error> {
    let client = match pool.get().await {
        Ok(client) => client,
        Err(err) => {
            return Err(actix_web::error::ErrorInternalServerError(format!("Failed to get database connection: {}", err)));
        }
    };

    let query = "SELECT COUNT(*) FROM invitation_code WHERE code = $1";
    let row = match client.query(query, &[&target_code]).await {
        Ok(rows) => rows.into_iter().next().unwrap(),
        Err(err) => {
            return Err(actix_web::error::ErrorInternalServerError(format!("Failed to execute query: {}", err)));
        }
    };

    let count: i64 = row.get(0);
    Ok(count > 0)
}