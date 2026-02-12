//! Role repository
//!
//! Database operations for roles.
//! Mirrors: app/models/role.rb

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use op_core::traits::Id;
use sqlx::{FromRow, PgPool};

use crate::repository::{Repository, RepositoryError, RepositoryResult};

/// Built-in role constants
pub mod builtin {
    pub const NON_BUILTIN: i32 = 0;
    pub const NON_MEMBER: i32 = 1;
    pub const ANONYMOUS: i32 = 2;
    pub const WORK_PACKAGE_VIEWER: i32 = 3;
    pub const WORK_PACKAGE_COMMENTER: i32 = 4;
    pub const WORK_PACKAGE_EDITOR: i32 = 5;
    pub const PROJECT_QUERY_VIEW: i32 = 6;
    pub const PROJECT_QUERY_EDIT: i32 = 7;
    pub const STANDARD_GLOBAL: i32 = 8;
}

/// Role database entity
#[derive(Debug, Clone, FromRow)]
pub struct RoleRow {
    pub id: i64,
    pub name: String,
    pub position: i32,
    pub builtin: i32,
    #[sqlx(rename = "type")]
    pub role_type: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl RoleRow {
    /// Check if this is a built-in role
    pub fn is_builtin(&self) -> bool {
        self.builtin != builtin::NON_BUILTIN
    }

    /// Check if this role can be assigned to members
    pub fn is_member(&self) -> bool {
        !self.is_builtin()
    }

    /// Check if this role can be deleted
    pub fn is_deletable(&self) -> bool {
        !self.is_builtin()
    }
}

/// DTO for creating a role
#[derive(Debug, Clone)]
pub struct CreateRoleDto {
    pub name: String,
    pub position: Option<i32>,
    pub role_type: String,
}

/// DTO for updating a role
#[derive(Debug, Clone, Default)]
pub struct UpdateRoleDto {
    pub name: Option<String>,
    pub position: Option<i32>,
}

/// Role permission entry
#[derive(Debug, Clone, FromRow)]
pub struct RolePermissionRow {
    pub id: i64,
    pub role_id: i64,
    pub permission: String,
}

/// Role repository implementation
pub struct RoleRepository {
    pool: PgPool,
}

impl RoleRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Find visible roles (excludes hidden role types)
    pub async fn find_visible(&self) -> RepositoryResult<Vec<RoleRow>> {
        let rows = sqlx::query_as::<_, RoleRow>(
            r#"
            SELECT id, name, position, builtin, type, created_at, updated_at
            FROM roles
            WHERE type IS NULL OR type NOT IN ('WorkPackageRole', 'ProjectQueryRole')
            ORDER BY builtin ASC, position ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Find givable roles (can be assigned to members)
    pub async fn find_givable(&self) -> RepositoryResult<Vec<RoleRow>> {
        let rows = sqlx::query_as::<_, RoleRow>(
            r#"
            SELECT id, name, position, builtin, type, created_at, updated_at
            FROM roles
            WHERE builtin = $1
            ORDER BY position ASC
            "#,
        )
        .bind(builtin::NON_BUILTIN)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Find builtin roles
    pub async fn find_builtin(&self) -> RepositoryResult<Vec<RoleRow>> {
        let rows = sqlx::query_as::<_, RoleRow>(
            r#"
            SELECT id, name, position, builtin, type, created_at, updated_at
            FROM roles
            WHERE builtin != $1
            ORDER BY builtin ASC, position ASC
            "#,
        )
        .bind(builtin::NON_BUILTIN)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Find non-member role
    pub async fn find_non_member(&self) -> RepositoryResult<Option<RoleRow>> {
        let row = sqlx::query_as::<_, RoleRow>(
            r#"
            SELECT id, name, position, builtin, type, created_at, updated_at
            FROM roles
            WHERE builtin = $1
            LIMIT 1
            "#,
        )
        .bind(builtin::NON_MEMBER)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    /// Find anonymous role
    pub async fn find_anonymous(&self) -> RepositoryResult<Option<RoleRow>> {
        let row = sqlx::query_as::<_, RoleRow>(
            r#"
            SELECT id, name, position, builtin, type, created_at, updated_at
            FROM roles
            WHERE builtin = $1
            LIMIT 1
            "#,
        )
        .bind(builtin::ANONYMOUS)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    /// Find role by name
    pub async fn find_by_name(&self, name: &str) -> RepositoryResult<Option<RoleRow>> {
        let row = sqlx::query_as::<_, RoleRow>(
            r#"
            SELECT id, name, position, builtin, type, created_at, updated_at
            FROM roles
            WHERE LOWER(name) = LOWER($1)
            "#,
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    /// Get permissions for a role
    pub async fn get_permissions(&self, role_id: Id) -> RepositoryResult<Vec<String>> {
        let permissions = sqlx::query_scalar::<_, String>(
            "SELECT permission FROM role_permissions WHERE role_id = $1",
        )
        .bind(role_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(permissions)
    }

    /// Add permission to a role
    pub async fn add_permission(&self, role_id: Id, permission: &str) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO role_permissions (role_id, permission, created_at, updated_at)
            VALUES ($1, $2, NOW(), NOW())
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(role_id)
        .bind(permission)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Remove permission from a role
    pub async fn remove_permission(&self, role_id: Id, permission: &str) -> RepositoryResult<()> {
        sqlx::query("DELETE FROM role_permissions WHERE role_id = $1 AND permission = $2")
            .bind(role_id)
            .bind(permission)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Set all permissions for a role
    pub async fn set_permissions(&self, role_id: Id, permissions: &[String]) -> RepositoryResult<()> {
        // Remove all existing permissions
        sqlx::query("DELETE FROM role_permissions WHERE role_id = $1")
            .bind(role_id)
            .execute(&self.pool)
            .await?;

        // Add new permissions
        for permission in permissions {
            self.add_permission(role_id, permission).await?;
        }

        Ok(())
    }

    /// Check if name is unique
    pub async fn is_name_unique(&self, name: &str, exclude_id: Option<Id>) -> RepositoryResult<bool> {
        let query = match exclude_id {
            Some(id) => sqlx::query_scalar::<_, bool>(
                "SELECT NOT EXISTS(SELECT 1 FROM roles WHERE LOWER(name) = LOWER($1) AND id != $2)",
            )
            .bind(name)
            .bind(id),
            None => sqlx::query_scalar::<_, bool>(
                "SELECT NOT EXISTS(SELECT 1 FROM roles WHERE LOWER(name) = LOWER($1))",
            )
            .bind(name),
        };

        let unique = query.fetch_one(&self.pool).await?;
        Ok(unique)
    }

    /// Get max position for ordering
    async fn get_max_position(&self) -> RepositoryResult<i32> {
        let max_pos = sqlx::query_scalar::<_, Option<i32>>("SELECT MAX(position) FROM roles")
            .fetch_one(&self.pool)
            .await?
            .unwrap_or(0);

        Ok(max_pos)
    }

    /// Check if role has any members
    async fn has_members(&self, role_id: Id) -> RepositoryResult<bool> {
        let has = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM member_roles WHERE role_id = $1)",
        )
        .bind(role_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(has)
    }
}

#[async_trait]
impl Repository<RoleRow, CreateRoleDto, UpdateRoleDto> for RoleRepository {
    async fn find_by_id(&self, id: Id) -> RepositoryResult<Option<RoleRow>> {
        let row = sqlx::query_as::<_, RoleRow>(
            r#"
            SELECT id, name, position, builtin, type, created_at, updated_at
            FROM roles
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    async fn find_all(&self, limit: i64, offset: i64) -> RepositoryResult<Vec<RoleRow>> {
        let rows = sqlx::query_as::<_, RoleRow>(
            r#"
            SELECT id, name, position, builtin, type, created_at, updated_at
            FROM roles
            ORDER BY builtin ASC, position ASC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM roles")
            .fetch_one(&self.pool)
            .await?;

        Ok(count)
    }

    async fn create(&self, dto: CreateRoleDto) -> RepositoryResult<RoleRow> {
        // Check name uniqueness
        if !self.is_name_unique(&dto.name, None).await? {
            return Err(RepositoryError::Conflict(
                "Role name has already been taken".to_string(),
            ));
        }

        let position = match dto.position {
            Some(pos) => pos,
            None => self.get_max_position().await? + 1,
        };

        let row = sqlx::query_as::<_, RoleRow>(
            r#"
            INSERT INTO roles (
                name, position, builtin, type, created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, NOW(), NOW()
            )
            RETURNING id, name, position, builtin, type, created_at, updated_at
            "#,
        )
        .bind(&dto.name)
        .bind(position)
        .bind(builtin::NON_BUILTIN)
        .bind(&dto.role_type)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    async fn update(&self, id: Id, dto: UpdateRoleDto) -> RepositoryResult<RoleRow> {
        // Check if role is builtin
        let existing = self.find_by_id(id).await?;
        if let Some(ref role) = existing {
            if role.is_builtin() {
                return Err(RepositoryError::Conflict(
                    "Cannot modify built-in roles".to_string(),
                ));
            }
        }

        // Check name uniqueness if changing
        if let Some(ref name) = dto.name {
            if !self.is_name_unique(name, Some(id)).await? {
                return Err(RepositoryError::Conflict(
                    "Role name has already been taken".to_string(),
                ));
            }
        }

        let row = sqlx::query_as::<_, RoleRow>(
            r#"
            UPDATE roles SET
                name = COALESCE($1, name),
                position = COALESCE($2, position),
                updated_at = NOW()
            WHERE id = $3
            RETURNING id, name, position, builtin, type, created_at, updated_at
            "#,
        )
        .bind(&dto.name)
        .bind(dto.position)
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| RepositoryError::NotFound(format!("Role with id {} not found", id)))?;

        Ok(row)
    }

    async fn delete(&self, id: Id) -> RepositoryResult<()> {
        // Check if role is builtin
        let existing = self.find_by_id(id).await?;
        if let Some(role) = existing {
            if role.is_builtin() {
                return Err(RepositoryError::Conflict(
                    "Cannot delete built-in roles".to_string(),
                ));
            }
        }

        // Check if role has members
        if self.has_members(id).await? {
            return Err(RepositoryError::Conflict(
                "Cannot delete role: members are using this role".to_string(),
            ));
        }

        let result = sqlx::query("DELETE FROM roles WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound(format!(
                "Role with id {} not found",
                id
            )));
        }

        Ok(())
    }

    async fn exists(&self, id: Id) -> RepositoryResult<bool> {
        let exists =
            sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM roles WHERE id = $1)")
                .bind(id)
                .fetch_one(&self.pool)
                .await?;

        Ok(exists)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_is_builtin() {
        let role = RoleRow {
            id: 1,
            name: "Non-member".to_string(),
            position: 1,
            builtin: builtin::NON_MEMBER,
            role_type: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert!(role.is_builtin());
        assert!(!role.is_member());
        assert!(!role.is_deletable());
    }

    #[test]
    fn test_role_is_member() {
        let role = RoleRow {
            id: 1,
            name: "Manager".to_string(),
            position: 1,
            builtin: builtin::NON_BUILTIN,
            role_type: Some("Role".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert!(!role.is_builtin());
        assert!(role.is_member());
        assert!(role.is_deletable());
    }
}
