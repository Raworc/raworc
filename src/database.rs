use crate::models::{AppState, DatabaseError};
use crate::rbac::{Role, RoleBinding, ServiceAccount, SubjectType};
use chrono::Utc;
use std::sync::Arc;
use sqlx::{query, Row};
use uuid::Uuid;
use tracing::{info, error};

impl AppState {
    // RBAC Operations
    // Service Account operations
    pub async fn create_service_account(
        &self,
        user: &str,
        _workspace: Option<String>,
        pass_hash: &str,
        description: Option<String>,
    ) -> Result<ServiceAccount, DatabaseError> {
        let id = Uuid::new_v4();
        let created_at = Utc::now().to_rfc3339();
        
        query(
            r#"
            INSERT INTO service_accounts (id, name, password_hash, description)
            VALUES ($1, $2, $3, $4)
            "#
        )
        .bind(id)
        .bind(user)
        .bind(pass_hash)
        .bind(&description)
        .execute(&*self.db)
        .await?;

        Ok(ServiceAccount {
            id: Some(id),
            user: user.to_string(),
            pass_hash: pass_hash.to_string(),
            description,
            created_at: created_at.clone(),
            updated_at: created_at,
            active: true,
            last_login_at: None,
        })
    }

    pub async fn get_service_account(
        &self,
        user: &str,
    ) -> Result<Option<ServiceAccount>, DatabaseError> {
        let row = query(
            r#"
            SELECT id, name, password_hash, description, created_at, updated_at, active, last_login_at
            FROM service_accounts
            WHERE name = $1
            "#
        )
        .bind(user)
        .fetch_optional(&*self.db)
        .await?;

        Ok(row.map(|r| ServiceAccount {
            id: Some(r.get("id")),
            user: r.get("name"),
            pass_hash: r.get("password_hash"),
            description: r.get("description"),
            created_at: r.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_rfc3339(),
            updated_at: r.get::<chrono::DateTime<chrono::Utc>, _>("updated_at").to_rfc3339(),
            active: r.get("active"),
            last_login_at: r.get::<Option<chrono::DateTime<chrono::Utc>>, _>("last_login_at")
                .map(|dt| dt.to_rfc3339()),
        }))
    }

    pub async fn get_all_service_accounts(&self) -> Result<Vec<ServiceAccount>, DatabaseError> {
        let rows = query(
            r#"
            SELECT id, name, password_hash, description, created_at, updated_at, active, last_login_at
            FROM service_accounts
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(&*self.db)
        .await?;

        Ok(rows.into_iter().map(|r| ServiceAccount {
            id: Some(r.get("id")),
            user: r.get("name"),
            pass_hash: r.get("password_hash"),
            description: r.get("description"),
            created_at: r.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_rfc3339(),
            updated_at: r.get::<chrono::DateTime<chrono::Utc>, _>("updated_at").to_rfc3339(),
            active: r.get("active"),
            last_login_at: r.get::<Option<chrono::DateTime<chrono::Utc>>, _>("last_login_at")
                .map(|dt| dt.to_rfc3339()),
        }).collect())
    }

    pub async fn delete_service_account(
        &self,
        user: &str,
    ) -> Result<bool, DatabaseError> {
        let result = query(
            r#"
            DELETE FROM service_accounts
            WHERE name = $1
            "#
        )
        .bind(user)
        .execute(&*self.db)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn delete_service_account_by_id(&self, id: &str) -> Result<bool, DatabaseError> {
        let uuid = Uuid::parse_str(id)?;
        
        let result = query(
            r#"
            DELETE FROM service_accounts
            WHERE id = $1
            "#
        )
        .bind(uuid)
        .execute(&*self.db)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn update_service_account_password(
        &self,
        user: &str,
        new_pass_hash: &str,
    ) -> Result<bool, DatabaseError> {
        let result = query(
            r#"
            UPDATE service_accounts
            SET password_hash = $1, updated_at = NOW()
            WHERE name = $2
            "#
        )
        .bind(new_pass_hash)
        .bind(user)
        .execute(&*self.db)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn update_service_account_password_by_id(
        &self,
        id: &str,
        new_pass_hash: &str,
    ) -> Result<bool, DatabaseError> {
        let uuid = Uuid::parse_str(id)?;
        
        let result = query(
            r#"
            UPDATE service_accounts
            SET password_hash = $1, updated_at = NOW()
            WHERE id = $2
            "#
        )
        .bind(new_pass_hash)
        .bind(uuid)
        .execute(&*self.db)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn update_service_account(
        &self,
        id: &str,
        workspace: Option<String>,
        description: Option<String>,
        active: Option<bool>,
    ) -> Result<bool, DatabaseError> {
        let uuid = Uuid::parse_str(id)?;
        
        // Build dynamic update query based on provided fields
        let result = if let (Some(ns), Some(desc), Some(act)) = (&workspace, &description, &active) {
            query(
                r#"
                UPDATE service_accounts
                SET workspace = $1, description = $2, active = $3, updated_at = NOW()
                WHERE id = $4
                "#
            )
            .bind(ns)
            .bind(desc)
            .bind(act)
            .bind(uuid)
            .execute(&*self.db)
            .await?
        } else if let (Some(ns), Some(desc)) = (&workspace, &description) {
            query(
                r#"
                UPDATE service_accounts
                SET workspace = $1, description = $2, updated_at = NOW()
                WHERE id = $3
                "#
            )
            .bind(ns)
            .bind(desc)
            .bind(uuid)
            .execute(&*self.db)
            .await?
        } else if let (Some(ns), Some(act)) = (&workspace, &active) {
            query(
                r#"
                UPDATE service_accounts
                SET workspace = $1, active = $2, updated_at = NOW()
                WHERE id = $3
                "#
            )
            .bind(ns)
            .bind(act)
            .bind(uuid)
            .execute(&*self.db)
            .await?
        } else if let (Some(desc), Some(act)) = (&description, &active) {
            query(
                r#"
                UPDATE service_accounts
                SET description = $1, active = $2, updated_at = NOW()
                WHERE id = $3
                "#
            )
            .bind(desc)
            .bind(act)
            .bind(uuid)
            .execute(&*self.db)
            .await?
        } else if let Some(ns) = workspace {
            query(
                r#"
                UPDATE service_accounts
                SET workspace = $1, updated_at = NOW()
                WHERE id = $2
                "#
            )
            .bind(ns)
            .bind(uuid)
            .execute(&*self.db)
            .await?
        } else if let Some(desc) = description {
            query(
                r#"
                UPDATE service_accounts
                SET description = $1, updated_at = NOW()
                WHERE id = $2
                "#
            )
            .bind(desc)
            .bind(uuid)
            .execute(&*self.db)
            .await?
        } else if let Some(act) = active {
            query(
                r#"
                UPDATE service_accounts
                SET active = $1, updated_at = NOW()
                WHERE id = $2
                "#
            )
            .bind(act)
            .bind(uuid)
            .execute(&*self.db)
            .await?
        } else {
            // No fields to update
            return Ok(false);
        };
        
        Ok(result.rows_affected() > 0)
    }

    pub async fn update_last_login(
        &self,
        user: &str,
    ) -> Result<bool, DatabaseError> {
        let result = query(
            r#"
            UPDATE service_accounts
            SET last_login_at = NOW()
            WHERE name = $1
            "#
        )
        .bind(user)
        .execute(&*self.db)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    // Role operations
    pub async fn create_role(&self, role: &Role) -> Result<Role, DatabaseError> {
        let id = Uuid::new_v4();
        let rules_json = serde_json::to_value(&role.rules)?;
        
        query(
            r#"
            INSERT INTO roles (id, name, rules, description)
            VALUES ($1, $2, $3, $4)
            "#
        )
        .bind(id)
        .bind(&role.name)
        .bind(&rules_json)
        .bind(&role.description)
        .execute(&*self.db)
        .await?;

        Ok(Role {
            id: Some(id),
            ..role.clone()
        })
    }

    pub async fn get_role(
        &self,
        name: &str,
    ) -> Result<Option<Role>, DatabaseError> {
        let row = query(
            r#"
            SELECT id, name, rules, description, created_at
            FROM roles
            WHERE name = $1
            "#
        )
        .bind(name)
        .fetch_optional(&*self.db)
        .await?;

        Ok(row.map(|r| Role {
            id: Some(r.get("id")),
            name: r.get("name"),
            rules: serde_json::from_value(r.get("rules")).unwrap_or_default(),
            description: r.get("description"),
            created_at: r.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_rfc3339(),
        }))
    }

    pub async fn get_all_roles(&self) -> Result<Vec<Role>, DatabaseError> {
        let rows = query(
            r#"
            SELECT id, name, rules, description, created_at
            FROM roles
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(&*self.db)
        .await?;

        Ok(rows.into_iter().map(|r| Role {
            id: Some(r.get("id")),
            name: r.get("name"),
            rules: serde_json::from_value(r.get("rules")).unwrap_or_default(),
            description: r.get("description"),
            created_at: r.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_rfc3339(),
        }).collect())
    }

    pub async fn delete_role(
        &self,
        name: &str,
    ) -> Result<bool, DatabaseError> {
        let result = query(
            r#"
            DELETE FROM roles
            WHERE name = $1
            "#
        )
        .bind(name)
        .execute(&*self.db)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    // Role Binding operations
    pub async fn create_role_binding(
        &self,
        role_binding: &RoleBinding,
    ) -> Result<RoleBinding, DatabaseError> {
        let id = Uuid::new_v4();
        
        // Convert SubjectType enum to string for database
        let principal_type_str = match role_binding.principal_type {
            SubjectType::ServiceAccount => "ServiceAccount",
            SubjectType::Subject => "User",
        };
        
        query(
            r#"
            INSERT INTO role_bindings (id, role_name, principal_name, principal_type, workspace)
            VALUES ($1, $2, $3, $4, $5)
            "#
        )
        .bind(id)
        .bind(&role_binding.role_name)
        .bind(&role_binding.principal_name)
        .bind(principal_type_str)
        .bind(&role_binding.workspace)
        .execute(&*self.db)
        .await?;

        Ok(RoleBinding {
            id: Some(id),
            ..role_binding.clone()
        })
    }

    pub async fn get_role_binding(
        &self,
        role_name: &str,
        workspace: Option<&str>,
    ) -> Result<Option<RoleBinding>, DatabaseError> {
        let row = query(
            r#"
            SELECT id, role_name, principal_name, principal_type, workspace, created_at
            FROM role_bindings
            WHERE role_name = $1 AND workspace IS NOT DISTINCT FROM $2
            LIMIT 1
            "#
        )
        .bind(role_name)
        .bind(workspace)
        .fetch_optional(&*self.db)
        .await?;

        Ok(row.map(|r| {
            let principal_type_str: String = r.get("principal_type");
            let principal_type = match principal_type_str.as_str() {
                "ServiceAccount" => SubjectType::ServiceAccount,
                _ => SubjectType::Subject,
            };
            
            RoleBinding {
                id: Some(r.get("id")),
                role_name: r.get("role_name"),
                principal_name: r.get("principal_name"),
                principal_type,
                workspace: r.get("workspace"),
                created_at: r.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_rfc3339(),
            }
        }))
    }

    pub async fn get_all_role_bindings(&self) -> Result<Vec<RoleBinding>, DatabaseError> {
        let rows = query(
            r#"
            SELECT id, role_name, principal_name, principal_type, workspace, created_at
            FROM role_bindings
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(&*self.db)
        .await?;

        Ok(rows.into_iter().map(|r| {
            let principal_type_str: String = r.get("principal_type");
            let principal_type = match principal_type_str.as_str() {
                "ServiceAccount" => SubjectType::ServiceAccount,
                _ => SubjectType::Subject,
            };
            
            RoleBinding {
                id: Some(r.get("id")),
                role_name: r.get("role_name"),
                principal_name: r.get("principal_name"),
                principal_type,
                workspace: r.get("workspace"),
                created_at: r.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_rfc3339(),
            }
        }).collect())
    }

    #[allow(dead_code)]
    pub async fn get_role_bindings_for_subject(
        &self,
        subject_name: &str,
        subject_type: SubjectType,
        workspace: Option<&str>,
    ) -> Result<Vec<RoleBinding>, DatabaseError> {
        let principal_type_str = match subject_type {
            SubjectType::Subject => "User",
            SubjectType::ServiceAccount => "ServiceAccount",
        };
        
        let rows = if let Some(ns) = workspace {
            query(
                r#"
                SELECT id, role_name, principal_name, principal_type, workspace, created_at
                FROM role_bindings
                WHERE principal_name = $1
                AND principal_type = $2
                AND (workspace = $3 OR workspace IS NULL)
                "#
            )
            .bind(subject_name)
            .bind(principal_type_str)
            .bind(ns)
            .fetch_all(&*self.db)
            .await?
        } else {
            query(
                r#"
                SELECT id, role_name, principal_name, principal_type, workspace, created_at
                FROM role_bindings
                WHERE principal_name = $1
                AND principal_type = $2
                "#
            )
            .bind(subject_name)
            .bind(principal_type_str)
            .fetch_all(&*self.db)
            .await?
        };

        Ok(rows.into_iter().map(|r| {
            let principal_type_str: String = r.get("principal_type");
            let principal_type = match principal_type_str.as_str() {
                "ServiceAccount" => SubjectType::ServiceAccount,
                _ => SubjectType::Subject,
            };
            
            RoleBinding {
                id: Some(r.get("id")),
                role_name: r.get("role_name"),
                principal_name: r.get("principal_name"),
                principal_type,
                workspace: r.get("workspace"),
                created_at: r.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_rfc3339(),
            }
        }).collect())
    }

    pub async fn delete_role_binding(
        &self,
        name: &str,
        workspace: Option<&str>,
    ) -> Result<bool, DatabaseError> {
        let result = query(
            r#"
            DELETE FROM role_bindings
            WHERE role_name = $1 AND workspace IS NOT DISTINCT FROM $2
            "#
        )
        .bind(name)
        .bind(workspace)
        .execute(&*self.db)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}

// Database connection utilities
pub async fn initialize_app_state(
    database_url: &str,
    jwt_secret: String,
) -> Result<AppState, Box<dyn std::error::Error>> {
    use sqlx::postgres::PgPoolOptions;
    
    let db = Arc::new(
        PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?,
    );

    // Run migrations
    info!("Running database migrations...");
    sqlx::migrate!("./migrations")
        .run(&*db)
        .await?;
    info!("Database migrations completed successfully");

    // Initialize Docker service (mandatory)
    info!("Initializing Docker service...");
    
    let docker_config = crate::docker::DockerSessionConfig {
        image: std::env::var("HOST_AGENT_IMAGE")
            .unwrap_or_else(|_| "python:3.11-slim".to_string()),
        cpu_limit: std::env::var("HOST_AGENT_CPU_LIMIT")
            .unwrap_or_else(|_| "0.5".to_string())
            .parse::<f64>()
            .unwrap_or(0.5),
        memory_limit: std::env::var("HOST_AGENT_MEMORY_LIMIT")
            .unwrap_or_else(|_| "536870912".to_string())  // 512MB in bytes
            .parse::<i64>()
            .unwrap_or(536870912),
        disk_limit: std::env::var("HOST_AGENT_DISK_LIMIT")
            .unwrap_or_else(|_| "1073741824".to_string())  // 1GB in bytes
            .parse::<i64>()
            .unwrap_or(1073741824),
        network: std::env::var("HOST_AGENT_NETWORK").ok(),
        volumes_path: std::env::var("HOST_AGENT_VOLUMES_PATH")
            .unwrap_or_else(|_| "/var/lib/raworc/volumes".to_string()),
    };
    
    // Create temporary AppState for Docker initialization
    let temp_app_state = AppState {
        db: db.clone(),
        jwt_secret: jwt_secret.clone(),
        docker: None,
    };
    
    let docker = match crate::docker::ContainerLifecycleManager::new(
        Arc::new(temp_app_state),
        docker_config,
    ).await {
        Ok(manager) => {
            info!("Docker service initialized successfully");
            
            // Start lifecycle manager
            let manager_arc = Arc::new(manager);
            let manager_clone = manager_arc.clone();
            tokio::spawn(async move {
                if let Err(e) = manager_clone.start().await {
                    error!("Failed to start Docker lifecycle manager: {}", e);
                }
            });
            
            info!("Docker lifecycle management started");
            Some(manager_arc)
        }
        Err(e) => {
            error!("Failed to initialize Docker service: {}", e);
            error!("Docker is required for raworc to function properly");
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Docker initialization failed: {}", e)
            )));
        }
    };

    Ok(AppState {
        db,
        jwt_secret,
        docker,
    })
}

// Database seeding for RBAC - only seeds if service_accounts table is empty
pub async fn seed_rbac_system(app_state: &AppState) -> Result<(), Box<dyn std::error::Error>> {
    use crate::rbac::{get_admin_role, RoleBinding, SubjectType};
    use bcrypt::{hash, DEFAULT_COST};
    use chrono::Utc;

    // Check if service_accounts table is empty
    let service_accounts = app_state.get_all_service_accounts().await?;
    if !service_accounts.is_empty() {
        info!("Service accounts already exist, skipping seeding");
        return Ok(());
    }

    info!("Service accounts table is empty, starting RBAC seeding...");

    // Create admin service account
    let admin_pass_hash = hash("admin", DEFAULT_COST)?;
    let _admin_service_account = app_state
        .create_service_account(
            "admin",
            None,
            &admin_pass_hash,
            Some("Default admin service account".to_string()),
        )
        .await?;
    info!("Admin service account created (user: admin, pass: admin)");

    // Create admin role
    let admin_role = get_admin_role();
    let _created_role = app_state.create_role(&admin_role).await?;
    info!("Admin role created");

    // Create admin role binding
    let admin_role_binding = RoleBinding {
        id: None,
        role_name: "admin".to_string(),
        principal_name: "admin".to_string(),
        principal_type: SubjectType::ServiceAccount,
        workspace: None, // Global access
        created_at: Utc::now().to_rfc3339(),
    };

    let _created_binding = app_state.create_role_binding(&admin_role_binding).await?;
    info!("Admin role binding created");

    Ok(())
}