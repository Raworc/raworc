use crate::models::{AppState, DatabaseError};
use crate::rbac::{Role, RoleBinding, ServiceAccount, SubjectType};
use chrono::Utc;
use std::sync::Arc;
use surrealdb::{
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
    Surreal,
};
use tracing::info;

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
        let service_account = ServiceAccount {
            id: None,
            user: user.to_string(),
            namespace,
            pass_hash: pass_hash.to_string(),
            description,
            created_at: Utc::now().to_rfc3339(),
            active: true,
        };

        let created: Option<ServiceAccount> = self
            .db
            .create("service_accounts")
            .content(service_account.clone())
            .await?;

        Ok(created.unwrap())
    }

    pub async fn get_service_account(
        &self,
        user: &str,
        namespace: Option<&str>,
    ) -> Result<Option<ServiceAccount>, DatabaseError> {
        let query = match namespace {
            Some(ns) => format!(
                "SELECT * FROM service_accounts WHERE user = '{user}' AND namespace = '{ns}'"
            ),
            None => format!(
                "SELECT * FROM service_accounts WHERE user = '{user}' AND namespace IS NONE"
            ),
        };

        let mut result = self.db.query(query).await?;
        let service_accounts: Vec<ServiceAccount> = result.take(0)?;
        Ok(service_accounts.into_iter().next())
    }

    pub async fn get_all_service_accounts(&self) -> Result<Vec<ServiceAccount>, DatabaseError> {
        let result: Vec<ServiceAccount> = self.db.select("service_accounts").await?;
        Ok(result)
    }

    pub async fn delete_service_account(
        &self,
        user: &str,
        namespace: Option<&str>,
    ) -> Result<bool, DatabaseError> {
        let query = match namespace {
            Some(ns) => {
                format!("DELETE FROM service_accounts WHERE user = '{user}' AND namespace = '{ns}'")
            }
            None => {
                format!("DELETE FROM service_accounts WHERE user = '{user}' AND namespace IS NONE")
            }
        };

        let result = self.db.query(query).await?;
        Ok(result.num_statements() > 0)
    }

    pub async fn delete_service_account_by_id(&self, id: &str) -> Result<bool, DatabaseError> {
        let query = format!("DELETE FROM service_accounts WHERE id = {id}");
        let result = self.db.query(query).await?;
        Ok(result.num_statements() > 0)
    }

    // Role operations
    pub async fn create_role(&self, role: &Role) -> Result<Role, DatabaseError> {
        let created: Option<Role> = self.db.create("roles").content(role.clone()).await?;

        Ok(created.unwrap())
    }

    pub async fn get_role(
        &self,
        name: &str,
        namespace: Option<&str>,
    ) -> Result<Option<Role>, DatabaseError> {
        let query = match namespace {
            Some(ns) => format!("SELECT * FROM roles WHERE name = '{name}' AND namespace = '{ns}'"),
            None => format!("SELECT * FROM roles WHERE name = '{name}' AND namespace IS NONE"),
        };

        let mut result = self.db.query(query).await?;
        let roles: Vec<Role> = result.take(0)?;
        Ok(roles.into_iter().next())
    }

    pub async fn get_all_roles(&self) -> Result<Vec<Role>, DatabaseError> {
        let result: Vec<Role> = self.db.select("roles").await?;
        Ok(result)
    }

    pub async fn delete_role(
        &self,
        name: &str,
        namespace: Option<&str>,
    ) -> Result<bool, DatabaseError> {
        let query = match namespace {
            Some(ns) => format!("DELETE FROM roles WHERE name = '{name}' AND namespace = '{ns}'"),
            None => format!("DELETE FROM roles WHERE name = '{name}' AND namespace IS NONE"),
        };

        let result = self.db.query(query).await?;
        Ok(result.num_statements() > 0)
    }

    // Role Binding operations
    pub async fn create_role_binding(
        &self,
        role_binding: &RoleBinding,
    ) -> Result<RoleBinding, DatabaseError> {
        let created: Option<RoleBinding> = self
            .db
            .create("role_bindings")
            .content(role_binding.clone())
            .await?;

        Ok(created.unwrap())
    }

    pub async fn get_role_binding(
        &self,
        name: &str,
        namespace: Option<&str>,
    ) -> Result<Option<RoleBinding>, DatabaseError> {
        let query = match namespace {
            Some(ns) => {
                format!("SELECT * FROM role_bindings WHERE name = '{name}' AND namespace = '{ns}'")
            }
            None => {
                format!("SELECT * FROM role_bindings WHERE name = '{name}' AND namespace IS NONE")
            }
        };

        let mut result = self.db.query(query).await?;
        let role_bindings: Vec<RoleBinding> = result.take(0)?;
        Ok(role_bindings.into_iter().next())
    }

    pub async fn get_all_role_bindings(&self) -> Result<Vec<RoleBinding>, DatabaseError> {
        let result: Vec<RoleBinding> = self.db.select("role_bindings").await?;
        Ok(result)
    }

    pub async fn get_role_bindings_for_subject(
        &self,
        subject_name: &str,
        subject_type: SubjectType,
        namespace: Option<&str>,
    ) -> Result<Vec<RoleBinding>, DatabaseError> {
        let query = match namespace {
            Some(ns) => format!(
                "SELECT * FROM role_bindings WHERE array::any(subjects, |$s| $s.name = '{}' AND $s.kind = '{}') AND (namespace = '{}' OR namespace IS NONE)",
                subject_name,
                match subject_type {
                    SubjectType::Subject => "Subject",
                    SubjectType::ServiceAccount => "ServiceAccount",
                },
                ns
            ),
            None => format!(
                "SELECT * FROM role_bindings WHERE array::any(subjects, |$s| $s.name = '{}' AND $s.kind = '{}')", 
                subject_name,
                match subject_type {
                    SubjectType::Subject => "Subject",
                    SubjectType::ServiceAccount => "ServiceAccount",
                }
            ),
        };

        let mut result = self.db.query(query).await?;
        let role_bindings: Vec<RoleBinding> = result.take(0)?;
        Ok(role_bindings)
    }

    pub async fn delete_role_binding(
        &self,
        name: &str,
        namespace: Option<&str>,
    ) -> Result<bool, DatabaseError> {
        let query = match namespace {
            Some(ns) => {
                format!("DELETE FROM role_bindings WHERE name = '{name}' AND namespace = '{ns}'")
            }
            None => {
                format!("DELETE FROM role_bindings WHERE name = '{name}' AND namespace IS NONE")
            }
        };

        let result = self.db.query(query).await?;
        Ok(result.num_statements() > 0)
    }
}

// Database connection utilities
pub async fn connect_to_surrealdb(
    url: &str,
    username: &str,
    password: &str,
    namespace: &str,
    database: &str,
) -> Result<Surreal<Client>, surrealdb::Error> {
    let db = Surreal::new::<Ws>(url).await?;

    db.signin(Root { username, password }).await?;

    db.use_ns(namespace).use_db(database).await?;

    Ok(db)
}

pub async fn initialize_app_state(
    db_url: &str,
    jwt_secret: String,
) -> Result<AppState, surrealdb::Error> {
    let db = connect_to_surrealdb(db_url, "root", "root", "raworc", "main").await?;

    Ok(AppState {
        db: Arc::new(db),
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
