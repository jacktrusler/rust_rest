#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
use serde::{Deserialize, Serialize};
use rocket_contrib::json::Json;
use rusqlite::Connection;

#[derive(Serialize)]
struct ToDoList {
    items: Vec<ToDoItem>,
}

#[derive(Serialize)]
struct ToDoItem {
    id: i64,
    item: String,
}

#[derive(Serialize)]
struct StatusMessage {
    message: String,
}

#[get("/")]
fn index() -> &'static str { //static lifetimes are for the entire duration of the program
    "it do be like that sometimes"
}

#[get("/todo")]
fn fetch_all_todo_items() -> Result<Json<ToDoList>, String> {
    let db_connection = match Connection::open("data.sqlite") {
        Ok(connection) => connection,
        Err(e) => {return Err(format!("Failed to connect to database, error code: {}", e))}
    };

    let mut statement = match db_connection.prepare("select id, item from todo_list;") {
        Ok(s) => s,
        Err(_) => return Err("Failed to prepare query".into()),
    };

    let results = statement.query_map([], |row| {
        Ok(ToDoItem {
            id: row.get(0)?,
            item: row.get(1)?,
        })
    });

    match results {
        Ok(rows) => {
            let collection: rusqlite::Result<Vec<_>> = rows.collect();

            match collection {
                Ok(items) => Ok(Json(ToDoList { items })),
                Err(_) => Err(format!("Could not collect items")),
            }
        }
        Err(_) => Err(String::from("Failed to fetch todo items")
        )
    }
}

#[post("/todo", format = "json", data = "<item>")]
fn add_todo_item(item: Json<String>) -> Result<Json<StatusMessage>, String> {
    let db_connection = match Connection::open("data.sqlite") {
        Ok(connection) => connection,
        Err(e) => {return Err(format!("Failed to connect to database, error code: {}", e))}
    };

    let mut statement = match db_connection.prepare("insert into todo_list (id, item) values (null, $1);") {
        Ok(s) => s,
        Err(_) => return Err("Failed to prepare query".into()),
    };
    let results = statement.execute(&[&item.0]);

    match results {
        Ok(rows_affected) => Ok(Json(StatusMessage {message: format!("{} rows inserted!", rows_affected)})),

        Err(_) => Err(String::from("Failed to insert todo items")
        )
    }
}

#[delete("/todo/<id>")]
fn delete_todo_item(id: i64) -> Result<Json<StatusMessage>, String> {
    let db_connection = match Connection::open("data.sqlite") {
        Ok(connection) => connection,
        Err(e) => {return Err(format!("Failed to connect to database, error code: {}", e))}
    };

    let mut statement = match db_connection.prepare("delete from todo_list where id = $1;") {
        Ok(s) => s,
        Err(_) => return Err("Failed to prepare query".into()),
    };
    let results = statement.execute(&[&id]);

    match results {
        Ok(rows_affected) => Ok(Json(StatusMessage {message: format!("{} rows deleted!", rows_affected)})),

        Err(_) => Err(String::from("Failed to insert todo items"))
    }
}

fn main() {
    let db_connection = Connection::open("data.sqlite").unwrap();

    db_connection.execute(
        "create table if not exists todo_list (
        id integer primary key,
        item varchar(64) not null
        );",
        [],
    ).unwrap();

    rocket::ignite().mount("/", routes![index, fetch_all_todo_items, add_todo_item, delete_todo_item]).launch();
}

