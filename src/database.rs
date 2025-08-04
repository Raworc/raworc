use crate::models::{AppState, DatabaseError};
use crate::rbac::{Role, RoleBinding, ServiceAccount, SubjectType};
use chrono::Utc;
use std::sync::Arc;
use sqlx::{query, Row};
use uuid::Uuid;
use tracing::info;
use serde_json::json;

impl AppState {
    // RBAC Operations
    // Service Account operations
    pub async fn create_service_account(
        &self,
        user: &str,
        namespace: Option<String>,
        pass_hash: &str,
        description: Option<String>,
    ) -> Result<ServiceAccount, DatabaseError> {
        let id = Uuid::new_v4();
        let created_at = Utc::now().to_rfc3339();
        let namespace_value = namespace.clone().unwrap_or_else(|| "default".to_string());
        
        query(
            r#"
            INSERT INTO service_accounts (id, name, namespace, password_hash, email)
            VALUES ($1, $2, $3, $4, $5)
            "#
        )
        .bind(id)
        .bind(user)
        .bind(&namespace_value)
        .bind(pass_hash)
        .bind(&description)
        .execute(&*self.db)
        .await?;

        Ok(ServiceAccount {
            id: Some(id),
            user: user.to_string(),
            namespace,
            pass_hash: pass_hash.to_string(),
            description,
            created_at,
            active: true,
        })
    }

    pub async fn get_service_account(
        &self,
        user: &str,
        namespace: Option<&str>,
    ) -> Result<Option<ServiceAccount>, DatabaseError> {
        let namespace_value = namespace.unwrap_or("default");
        
        let row = query(
            r#"
            SELECT id, name, namespace, password_hash, email, created_at
            FROM service_accounts
            WHERE name = $1 AND namespace = $2
            "#
        )
        .bind(user)
        .bind(namespace_value)
        .fetch_optional(&*self.db)
        .await?;

        Ok(row.map(|r| ServiceAccount {
            id: Some(r.get("id")),
            user: r.get("name"),
            namespace: {
                let ns: String = r.get("namespace");
                if ns == "default" { None } else { Some(ns) }
            },
            pass_hash: r.get("password_hash"),
            description: r.get("email"),
            created_at: r.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_rfc3339(),
            active: true,
        }))
    }

    pub async fn get_all_service_accounts(&self) -> Result<Vec<ServiceAccount>, DatabaseError> {
        let rows = query(
            r#"
            SELECT id, name, namespace, password_hash, email, created_at
            FROM service_accounts
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(&*self.db)
        .await?;

        Ok(rows.into_iter().map(|r| ServiceAccount {
            id: Some(r.get("id")),
            user: r.get("name"),
            namespace: {
                let ns: String = r.get("namespace");
                if ns == "default" { None } else { Some(ns) }
            },
            pass_hash: r.get("password_hash"),
            description: r.get("email"),
            created_at: r.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_rfc3339(),
            active: true,
        }).collect())
    }

    pub async fn delete_service_account(
        &self,
        user: &str,
        namespace: Option<&str>,
    ) -> Result<bool, DatabaseError> {
        let namespace_value = namespace.unwrap_or("default");
        
        let result = query(
            r#"
            DELETE FROM service_accounts
            WHERE name = $1 AND namespace = $2
            "#
        )
        .bind(user)
        .bind(namespace_value)
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

    // Role operations
    pub async fn create_role(&self, role: &Role) -> Result<Role, DatabaseError> {
        let id = Uuid::new_v4();
        let namespace_value = role.namespace.clone().unwrap_or_else(|| "default".to_string());
        let rules_json = serde_json::to_value(&role.rules)?;
        
        query(
            r#"
            INSERT INTO roles (id, name, namespace, rules)
            VALUES ($1, $2, $3, $4)
            "#
        )
        .bind(id)
        .bind(&role.name)
        .bind(&namespace_value)
        .bind(&rules_json)
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
        namespace: Option<&str>,
    ) -> Result<Option<Role>, DatabaseError> {
        let namespace_value = namespace.unwrap_or("default");
        
        let row = query(
            r#"
            SELECT id, name, namespace, rules, created_at
            FROM roles
            WHERE name = $1 AND namespace = $2
            "#
        )
        .bind(name)
        .bind(namespace_value)
        .fetch_optional(&*self.db)
        .await?;

        Ok(row.map(|r| Role {
            id: Some(r.get("id")),
            name: r.get("name"),
            namespace: {
                let ns: String = r.get("namespace");
                if ns == "default" { None } else { Some(ns) }
            },
            rules: serde_json::from_value(r.get("rules")).unwrap_or_default(),
            description: None,
            created_at: r.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_rfc3339(),
        }))
    }

    pub async fn get_all_roles(&self) -> Result<Vec<Role>, DatabaseError> {
        let rows = query(
            r#"
            SELECT id, name, namespace, rules, created_at
            FROM roles
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(&*self.db)
        .await?;

        Ok(rows.into_iter().map(|r| Role {
            id: Some(r.get("id")),
            name: r.get("name"),
            namespace: {
                let ns: String = r.get("namespace");
                if ns == "default" { None } else { Some(ns) }
            },
            rules: serde_json::from_value(r.get("rules")).unwrap_or_default(),
            description: None,
            created_at: r.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_rfc3339(),
        }).collect())
    }

    pub async fn delete_role(
        &self,
        name: &str,
        namespace: Option<&str>,
    ) -> Result<bool, DatabaseError> {
        let namespace_value = namespace.unwrap_or("default");
        
        let result = query(
            r#"
            DELETE FROM roles
            WHERE name = $1 AND namespace = $2
            "#
        )
        .bind(name)
        .bind(namespace_value)
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
        let namespace_value = role_binding.namespace.clone().unwrap_or_else(|| "default".to_string());
        let subjects_json = serde_json::to_value(&role_binding.subjects)?;
        
        query(
            r#"
            INSERT INTO role_bindings (id, name, namespace, role_name, subjects)
            VALUES ($1, $2, $3, $4, $5)
            "#
        )
        .bind(id)
        .bind(&role_binding.name)
        .bind(&namespace_value)
        .bind(&role_binding.role_ref.name)
        .bind(&subjects_json)
        .execute(&*self.db)
        .await?;

        Ok(RoleBinding {
            id: Some(id),
            ..role_binding.clone()
        })
    }

    pub async fn get_role_binding(
        &self,
        name: &str,
        namespace: Option<&str>,
    ) -> Result<Option<RoleBinding>, DatabaseError> {
        let namespace_value = namespace.unwrap_or("default");
        
        let row = query(
            r#"
            SELECT id, name, namespace, role_name, subjects, created_at
            FROM role_bindings
            WHERE name = $1 AND namespace = $2
            "#
        )
        .bind(name)
        .bind(namespace_value)
        .fetch_optional(&*self.db)
        .await?;

        Ok(row.map(|r| RoleBinding {
            id: Some(r.get("id")),
            name: r.get("name"),
            namespace: {
                let ns: String = r.get("namespace");
                if ns == "default" { None } else { Some(ns) }
            },
            role_ref: crate::rbac::RoleRef {
                kind: "Role".to_string(),
                name: r.get("role_name"),
                api_group: "rbac".to_string(),
            },
            subjects: serde_json::from_value(r.get("subjects")).unwrap_or_default(),
            created_at: r.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_rfc3339(),
        }))
    }

    pub async fn get_all_role_bindings(&self) -> Result<Vec<RoleBinding>, DatabaseError> {
        let rows = query(
            r#"
            SELECT id, name, namespace, role_name, subjects, created_at
            FROM role_bindings
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(&*self.db)
        .await?;

        Ok(rows.into_iter().map(|r| RoleBinding {
            id: Some(r.get("id")),
            name: r.get("name"),
            namespace: {
                let ns: String = r.get("namespace");
                if ns == "default" { None } else { Some(ns) }
            },
            role_ref: crate::rbac::RoleRef {
                kind: "Role".to_string(),
                name: r.get("role_name"),
                api_group: "rbac".to_string(),
            },
            subjects: serde_json::from_value(r.get("subjects")).unwrap_or_default(),
            created_at: r.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_rfc3339(),
        }).collect())
    }

    #[allow(dead_code)]
    pub async fn get_role_bindings_for_subject(
        &self,
        subject_name: &str,
        subject_type: SubjectType,
        namespace: Option<&str>,
    ) -> Result<Vec<RoleBinding>, DatabaseError> {
        let subject_kind = match subject_type {
            SubjectType::Subject => "Subject",
            SubjectType::ServiceAccount => "ServiceAccount",
        };
        
        let subject_filter = json!([{
            "name": subject_name,
            "kind": subject_kind
        }]);
        
        let rows = if let Some(ns) = namespace {
            query(
                r#"
                SELECT id, name, namespace, role_name, subjects, created_at
                FROM role_bindings
                WHERE (namespace = $1 OR namespace = 'default')
                AND subjects @> $2::jsonb
                "#
            )
            .bind(ns)
            .bind(&subject_filter)
            .fetch_all(&*self.db)
            .await?
        } else {
            query(
                r#"
                SELECT id, name, namespace, role_name, subjects, created_at
                FROM role_bindings
                WHERE subjects @> $1::jsonb
                "#
            )
            .bind(&subject_filter)
            .fetch_all(&*self.db)
            .await?
        };

        Ok(rows.into_iter().map(|r| RoleBinding {
            id: Some(r.get("id")),
            name: r.get("name"),
            namespace: {
                let ns: String = r.get("namespace");
                if ns == "default" { None } else { Some(ns) }
            },
            role_ref: crate::rbac::RoleRef {
                kind: "Role".to_string(),
                name: r.get("role_name"),
                api_group: "rbac".to_string(),
            },
            subjects: serde_json::from_value(r.get("subjects")).unwrap_or_default(),
            created_at: r.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_rfc3339(),
        }).collect())
    }

    pub async fn delete_role_binding(
        &self,
        name: &str,
        namespace: Option<&str>,
    ) -> Result<bool, DatabaseError> {
        let namespace_value = namespace.unwrap_or("default");
        
        let result = query(
            r#"
            DELETE FROM role_bindings
            WHERE name = $1 AND namespace = $2
            "#
        )
        .bind(name)
        .bind(namespace_value)
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

    Ok(AppState {
        db,
        jwt_secret,
    })
}

// Database seeding for RBAC - only seeds if service_accounts table is empty
pub async fn seed_rbac_system(app_state: &AppState) -> Result<(), Box<dyn std::error::Error>> {
    use crate::rbac::{get_admin_role, RoleBinding, RoleBindingSubject, RoleRef, SubjectType};
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
        name: "admin-binding".to_string(),
        namespace: None,
        role_ref: RoleRef {
            kind: "Role".to_string(),
            name: "admin".to_string(),
            api_group: "rbac".to_string(),
        },
        subjects: vec![RoleBindingSubject {
            kind: SubjectType::ServiceAccount,
            name: "admin".to_string(),
            namespace: None,
        }],
        created_at: Utc::now().to_rfc3339(),
    };

    let _created_binding = app_state.create_role_binding(&admin_role_binding).await?;
    info!("Admin role binding created");

    Ok(())
}