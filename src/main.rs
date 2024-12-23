use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use actix_cors::Cors;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Task {
    id: usize,
    description: String,
    completed: bool,
}

#[derive(Default)]
struct AppState {
    tasks: Mutex<Vec<Task>>,
    next_id: Mutex<usize>,
}

impl Task {
    fn new(id: usize, description: String) -> Self {
        Self {
            id,
            description,
            completed: false,
        }
    }
}

#[derive(Debug, Deserialize)]
struct CreateTask {
    description: String,
}

#[derive(Debug, Deserialize)]
struct UpdateTask {
    description: Option<String>,
    completed: Option<bool>,
}

async fn list_tasks(state: web::Data<AppState>) -> impl Responder {
    let tasks = state.tasks.lock().unwrap();
    HttpResponse::Ok().json(&*tasks)
}

async fn add_task(task: web::Json<CreateTask>, state: web::Data<AppState>) -> impl Responder {
    let mut tasks = state.tasks.lock().unwrap();
    let mut next_id = state.next_id.lock().unwrap();
    let id = *next_id;
    *next_id += 1;
    let new_task = Task::new(id, task.description.clone());
    tasks.push(new_task);
    HttpResponse::Created().finish()
}

async fn update_task(
    task_id: web::Path<usize>,
    task_update: web::Json<UpdateTask>,
    state: web::Data<AppState>,
) -> impl Responder {
    let mut tasks = state.tasks.lock().unwrap();
    if let Some(task) = tasks.iter_mut().find(|t| t.id == *task_id) {
        if let Some(description) = &task_update.description {
            task.description = description.clone();
        }
        if let Some(completed) = task_update.completed {
            task.completed = completed;
        }
        HttpResponse::Ok().finish()
    } else {
        HttpResponse::NotFound().body("Task not found")
    }
}

async fn delete_task(task_id: web::Path<usize>, state: web::Data<AppState>) -> impl Responder {
    let mut tasks = state.tasks.lock().unwrap();
    if let Some(pos) = tasks.iter().position(|t| t.id == *task_id) {
        tasks.remove(pos);
        HttpResponse::Ok().finish()
    } else {
        HttpResponse::NotFound().body("Task not found")
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let data = web::Data::new(AppState::default());

    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .app_data(data.clone())
            .route("/tasks", web::get().to(list_tasks))
            .route("/tasks", web::post().to(add_task))
            .route("/tasks/{id}", web::put().to(update_task))
            .route("/tasks/{id}", web::delete().to(delete_task))
    })
        .bind("127.0.0.1:1488")?
        .run()
        .await
}
