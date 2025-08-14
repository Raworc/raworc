use super::error::{HostError, Result};
use chrono::{Local, NaiveDate};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tracing::{debug, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Todo {
    pub id: usize,
    pub completed: bool,
    pub priority: Option<char>,
    pub creation_date: Option<NaiveDate>,
    pub completion_date: Option<NaiveDate>,
    pub description: String,
    pub contexts: Vec<String>,  // @context
    pub projects: Vec<String>,  // +project
}

impl Todo {
    fn from_line(id: usize, line: &str) -> Self {
        let mut line = line.to_string();
        let mut completed = false;
        let mut priority = None;
        let mut creation_date = None;
        let mut completion_date = None;
        
        // Check if completed (starts with "x ")
        if line.starts_with("x ") {
            completed = true;
            line = line[2..].to_string();
            
            // Check for completion date (YYYY-MM-DD format)
            if line.len() >= 10 && line.chars().nth(4) == Some('-') && line.chars().nth(7) == Some('-') {
                if let Ok(date) = NaiveDate::parse_from_str(&line[..10], "%Y-%m-%d") {
                    completion_date = Some(date);
                    line = line[11..].to_string();
                }
            }
        }
        
        // Check for priority (A) through (Z)
        if line.len() >= 4 && line.starts_with('(') && line.chars().nth(2) == Some(')') {
            if let Some(p) = line.chars().nth(1) {
                if p.is_ascii_uppercase() {
                    priority = Some(p);
                    line = line[4..].to_string();
                }
            }
        }
        
        // Check for creation date
        if line.len() >= 10 && line.chars().nth(4) == Some('-') && line.chars().nth(7) == Some('-') {
            if let Ok(date) = NaiveDate::parse_from_str(&line[..10], "%Y-%m-%d") {
                creation_date = Some(date);
                line = line[11..].to_string();
            }
        }
        
        // Extract contexts (@word)
        let mut contexts = Vec::new();
        for word in line.split_whitespace() {
            if word.starts_with('@') && word.len() > 1 {
                contexts.push(word[1..].to_string());
            }
        }
        
        // Extract projects (+word)
        let mut projects = Vec::new();
        for word in line.split_whitespace() {
            if word.starts_with('+') && word.len() > 1 {
                projects.push(word[1..].to_string());
            }
        }
        
        Todo {
            id,
            completed,
            priority,
            creation_date,
            completion_date,
            description: line.trim().to_string(),
            contexts,
            projects,
        }
    }
    
    fn to_line(&self) -> String {
        let mut line = String::new();
        
        if self.completed {
            line.push_str("x ");
            if let Some(date) = self.completion_date {
                line.push_str(&format!("{} ", date.format("%Y-%m-%d")));
            }
        }
        
        if let Some(priority) = self.priority {
            line.push_str(&format!("({}) ", priority));
        }
        
        if let Some(date) = self.creation_date {
            line.push_str(&format!("{} ", date.format("%Y-%m-%d")));
        }
        
        line.push_str(&self.description);
        
        line
    }
}

pub struct TodoManager {
    file_path: String,
    todos: Vec<Todo>,
}

impl TodoManager {
    pub async fn new(file_path: &str) -> Result<Self> {
        let mut manager = Self {
            file_path: file_path.to_string(),
            todos: Vec::new(),
        };
        
        // Create file if it doesn't exist
        if !Path::new(file_path).exists() {
            fs::write(file_path, "").await?;
            info!("Created todo.txt file at: {}", file_path);
        }
        
        manager.load().await?;
        Ok(manager)
    }
    
    pub async fn load(&mut self) -> Result<()> {
        let content = fs::read_to_string(&self.file_path).await?;
        
        self.todos = content
            .lines()
            .enumerate()
            .filter(|(_, line)| !line.trim().is_empty())
            .map(|(idx, line)| Todo::from_line(idx + 1, line))
            .collect();
        
        debug!("Loaded {} todos", self.todos.len());
        Ok(())
    }
    
    pub async fn save(&self) -> Result<()> {
        let content: String = self.todos
            .iter()
            .map(|todo| todo.to_line())
            .collect::<Vec<_>>()
            .join("\n");
        
        let mut file = fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&self.file_path)
            .await?;
        
        file.write_all(content.as_bytes()).await?;
        file.write_all(b"\n").await?;
        
        debug!("Saved {} todos", self.todos.len());
        Ok(())
    }
    
    pub async fn add(&mut self, description: String, priority: Option<char>) -> Result<Todo> {
        let id = self.todos.len() + 1;
        let todo = Todo {
            id,
            completed: false,
            priority,
            creation_date: Some(Local::now().date_naive()),
            completion_date: None,
            description,
            contexts: Vec::new(),
            projects: Vec::new(),
        };
        
        self.todos.push(todo.clone());
        self.save().await?;
        
        info!("Added todo #{}: {}", id, todo.description);
        Ok(todo)
    }
    
    pub async fn complete(&mut self, id: usize) -> Result<()> {
        if let Some(todo) = self.todos.iter_mut().find(|t| t.id == id) {
            todo.completed = true;
            todo.completion_date = Some(Local::now().date_naive());
            self.save().await?;
            info!("Completed todo #{}", id);
            Ok(())
        } else {
            Err(HostError::Todo(format!("Todo #{} not found", id)))
        }
    }
    
    pub async fn update(&mut self, id: usize, description: String) -> Result<()> {
        if let Some(todo) = self.todos.iter_mut().find(|t| t.id == id) {
            todo.description = description;
            self.save().await?;
            info!("Updated todo #{}", id);
            Ok(())
        } else {
            Err(HostError::Todo(format!("Todo #{} not found", id)))
        }
    }
    
    pub async fn list(&self, include_completed: bool) -> Vec<Todo> {
        if include_completed {
            self.todos.clone()
        } else {
            self.todos.iter()
                .filter(|t| !t.completed)
                .cloned()
                .collect()
        }
    }
    
    pub async fn get(&self, id: usize) -> Option<Todo> {
        self.todos.iter().find(|t| t.id == id).cloned()
    }
}