//! Members repository
//!
//! Mirrors: app/models/member.rb

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};

use crate::{Pagination, PaginatedResult, Repository, RepositoryError};

/// Member row from database
#[derive(Debug, Clone, FromRow)]
pub struct MemberRow {
    pub id: i64,
    pub user_id: i64,
    pub project_id: Option<i64>,
    pub entity_type: Option<String>,
    pub entity_id: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Member with roles (includes joined role data)
#[derive(Debug, Clone)]
pub struct MemberWithRoles {
    pub member: MemberRow,
    pub role_ids: Vec<i64>,
}

/// Member role row from database
#[derive(Debug, Clone, FromRow)]
pub struct MemberRoleRow {
    pub id: i64,
    pub member_id: i64,
    pub role_id: i64,
    pub inherited_from: Option<i64>,
}

/// DTO for creating a member
#[derive(Debug, Clone)]
pub struct CreateMemberDto {
    pub user_id: i64,
    pub project_id: Option<i64>,
    pub role_ids: Vec<i64>,
    pub entity_type: Option<String>,
    pub entity_id: Option<i64>,
}

/// DTO for updating a member
#[derive(Debug, Clone, Default)]
pub struct UpdateMemberDto {
    pub role_ids: Option<Vec<i64>>,
}

/// Member repository
pub struct MemberRepository {
    pool: PgPool,
}

impl MemberRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Find members by project
    pub async fn find_by_project(
        &self,
        project_id: i64,
        pagination: Pagination,
    ) -> Result<PaginatedResult<MemberWithRoles>, RepositoryError> {
        let members = sqlx::query_as::<_, MemberRow>(
            r#"
            SELECT id, user_id, project_id, entity_type, entity_id, created_at, updated_at
            FROM members
            WHERE project_id = $1 AND entity_type IS NULL
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(project_id)
        .bind(pagination.limit)
        .bind(pagination.offset)
        .fetch_all(&self.pool)
        .await?;

        let total = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM members WHERE project_id = $1 AND entity_type IS NULL",
        )
        .bind(project_id)
        .fetch_one(&self.pool)
        .await?;

        // Fetch roles for each member
        let mut result = Vec::with_capacity(members.len());
        for member in members {
            let role_ids = self.get_role_ids(member.id).await?;
            result.push(MemberWithRoles { member, role_ids });
        }

        Ok(PaginatedResult {
            items: result,
            total,
            limit: pagination.limit,
            offset: pagination.offset,
        })
    }

    /// Find members by user
    pub async fn find_by_user(&self, user_id: i64) -> Result<Vec<MemberWithRoles>, RepositoryError> {
        let members = sqlx::query_as::<_, MemberRow>(
            r#"
            SELECT id, user_id, project_id, entity_type, entity_id, created_at, updated_at
            FROM members
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        let mut result = Vec::with_capacity(members.len());
        for member in members {
            let role_ids = self.get_role_ids(member.id).await?;
            result.push(MemberWithRoles { member, role_ids });
        }

        Ok(result)
    }

    /// Find member by project and user
    pub async fn find_by_project_and_user(
        &self,
        project_id: i64,
        user_id: i64,
    ) -> Result<Option<MemberWithRoles>, RepositoryError> {
        let member = sqlx::query_as::<_, MemberRow>(
            r#"
            SELECT id, user_id, project_id, entity_type, entity_id, created_at, updated_at
            FROM members
            WHERE project_id = $1 AND user_id = $2 AND entity_type IS NULL
            "#,
        )
        .bind(project_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        match member {
            Some(m) => {
                let role_ids = self.get_role_ids(m.id).await?;
                Ok(Some(MemberWithRoles { member: m, role_ids }))
            }
            None => Ok(None),
        }
    }

    /// Find global members (not scoped to a project)
    pub async fn find_global(&self) -> Result<Vec<MemberWithRoles>, RepositoryError> {
        let members = sqlx::query_as::<_, MemberRow>(
            r#"
            SELECT id, user_id, project_id, entity_type, entity_id, created_at, updated_at
            FROM members
            WHERE project_id IS NULL AND entity_type IS NULL
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut result = Vec::with_capacity(members.len());
        for member in members {
            let role_ids = self.get_role_ids(member.id).await?;
            result.push(MemberWithRoles { member, role_ids });
        }

        Ok(result)
    }

    /// Get role IDs for a member
    async fn get_role_ids(&self, member_id: i64) -> Result<Vec<i64>, RepositoryError> {
        let role_ids = sqlx::query_scalar::<_, i64>(
            "SELECT DISTINCT role_id FROM member_roles WHERE member_id = $1 ORDER BY role_id",
        )
        .bind(member_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(role_ids)
    }

    /// Set roles for a member (replaces existing non-inherited roles)
    async fn set_roles(&self, member_id: i64, role_ids: &[i64]) -> Result<(), RepositoryError> {
        // Delete existing non-inherited roles
        sqlx::query("DELETE FROM member_roles WHERE member_id = $1 AND inherited_from IS NULL")
            .bind(member_id)
            .execute(&self.pool)
            .await?;

        // Insert new roles
        for role_id in role_ids {
            sqlx::query(
                "INSERT INTO member_roles (member_id, role_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
            )
            .bind(member_id)
            .bind(role_id)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    /// Find member by ID with roles
    pub async fn find_by_id_with_roles(
        &self,
        id: i64,
    ) -> Result<Option<MemberWithRoles>, RepositoryError> {
        let member = sqlx::query_as::<_, MemberRow>(
            r#"
            SELECT id, user_id, project_id, entity_type, entity_id, created_at, updated_at
            FROM members
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        match member {
            Some(m) => {
                let role_ids = self.get_role_ids(m.id).await?;
                Ok(Some(MemberWithRoles { member: m, role_ids }))
            }
            None => Ok(None),
        }
    }

    /// Find all members with roles
    pub async fn find_all_with_roles(
        &self,
        pagination: Pagination,
    ) -> Result<PaginatedResult<MemberWithRoles>, RepositoryError> {
        let members = sqlx::query_as::<_, MemberRow>(
            r#"
            SELECT id, user_id, project_id, entity_type, entity_id, created_at, updated_at
            FROM members
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(pagination.limit)
        .bind(pagination.offset)
        .fetch_all(&self.pool)
        .await?;

        let total = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM members")
            .fetch_one(&self.pool)
            .await?;

        let mut result = Vec::with_capacity(members.len());
        for member in members {
            let role_ids = self.get_role_ids(member.id).await?;
            result.push(MemberWithRoles { member, role_ids });
        }

        Ok(PaginatedResult {
            items: result,
            total,
            limit: pagination.limit,
            offset: pagination.offset,
        })
    }

    /// Check if a membership already exists
    pub async fn membership_exists(
        &self,
        user_id: i64,
        project_id: Option<i64>,
        entity_type: Option<&str>,
        entity_id: Option<i64>,
    ) -> Result<bool, RepositoryError> {
        let count = match (project_id, entity_type, entity_id) {
            (Some(pid), Some(et), Some(eid)) => {
                sqlx::query_scalar::<_, i64>(
                    r#"
                    SELECT COUNT(*) FROM members
                    WHERE user_id = $1 AND project_id = $2 AND entity_type = $3 AND entity_id = $4
                    "#,
                )
                .bind(user_id)
                .bind(pid)
                .bind(et)
                .bind(eid)
                .fetch_one(&self.pool)
                .await?
            }
            (Some(pid), None, None) => {
                sqlx::query_scalar::<_, i64>(
                    r#"
                    SELECT COUNT(*) FROM members
                    WHERE user_id = $1 AND project_id = $2 AND entity_type IS NULL
                    "#,
                )
                .bind(user_id)
                .bind(pid)
                .fetch_one(&self.pool)
                .await?
            }
            (None, None, None) => {
                sqlx::query_scalar::<_, i64>(
                    r#"
                    SELECT COUNT(*) FROM members
                    WHERE user_id = $1 AND project_id IS NULL AND entity_type IS NULL
                    "#,
                )
                .bind(user_id)
                .fetch_one(&self.pool)
                .await?
            }
            _ => return Err(RepositoryError::Validation("Invalid membership parameters".to_string())),
        };

        Ok(count > 0)
    }
}

#[async_trait]
impl Repository<MemberRow, CreateMemberDto, UpdateMemberDto> for MemberRepository {
    async fn find_by_id(&self, id: i64) -> Result<Option<MemberRow>, RepositoryError> {
        let row = sqlx::query_as::<_, MemberRow>(
            r#"
            SELECT id, user_id, project_id, entity_type, entity_id, created_at, updated_at
            FROM members
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    async fn find_all(&self, limit: i64, offset: i64) -> Result<Vec<MemberRow>, RepositoryError> {
        let rows = sqlx::query_as::<_, MemberRow>(
            r#"
            SELECT id, user_id, project_id, entity_type, entity_id, created_at, updated_at
            FROM members
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    async fn count(&self) -> Result<i64, RepositoryError> {
        let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM members")
            .fetch_one(&self.pool)
            .await?;

        Ok(count)
    }

    async fn exists(&self, id: i64) -> Result<bool, RepositoryError> {
        let count =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM members WHERE id = $1")
                .bind(id)
                .fetch_one(&self.pool)
                .await?;

        Ok(count > 0)
    }

    async fn create(&self, dto: CreateMemberDto) -> Result<MemberRow, RepositoryError> {
        // Validate role_ids not empty
        if dto.role_ids.is_empty() {
            return Err(RepositoryError::Validation(
                "Roles can't be blank".to_string(),
            ));
        }

        // Check if membership already exists
        if self
            .membership_exists(
                dto.user_id,
                dto.project_id,
                dto.entity_type.as_deref(),
                dto.entity_id,
            )
            .await?
        {
            return Err(RepositoryError::Conflict(
                "Member already exists for this user/project combination".to_string(),
            ));
        }

        // Create member
        let row = sqlx::query_as::<_, MemberRow>(
            r#"
            INSERT INTO members (user_id, project_id, entity_type, entity_id, created_at, updated_at)
            VALUES ($1, $2, $3, $4, NOW(), NOW())
            RETURNING id, user_id, project_id, entity_type, entity_id, created_at, updated_at
            "#,
        )
        .bind(dto.user_id)
        .bind(dto.project_id)
        .bind(&dto.entity_type)
        .bind(dto.entity_id)
        .fetch_one(&self.pool)
        .await?;

        // Set roles
        self.set_roles(row.id, &dto.role_ids).await?;

        Ok(row)
    }

    async fn update(&self, id: i64, dto: UpdateMemberDto) -> Result<MemberRow, RepositoryError> {
        // Verify member exists
        let existing = self
            .find_by_id(id)
            .await?
            .ok_or_else(|| RepositoryError::NotFound(format!("Member {} not found", id)))?;

        // Update roles if provided
        if let Some(role_ids) = dto.role_ids {
            if role_ids.is_empty() {
                return Err(RepositoryError::Validation(
                    "Roles can't be blank".to_string(),
                ));
            }
            self.set_roles(id, &role_ids).await?;
        }

        // Update timestamp
        let row = sqlx::query_as::<_, MemberRow>(
            r#"
            UPDATE members
            SET updated_at = NOW()
            WHERE id = $1
            RETURNING id, user_id, project_id, entity_type, entity_id, created_at, updated_at
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    async fn delete(&self, id: i64) -> Result<(), RepositoryError> {
        // Check if member exists
        if !self.exists(id).await? {
            return Err(RepositoryError::NotFound(format!(
                "Member {} not found",
                id
            )));
        }

        // Check if member has only inherited roles (can't delete)
        let non_inherited_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM member_roles WHERE member_id = $1 AND inherited_from IS NULL",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        let total_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM member_roles WHERE member_id = $1",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        if non_inherited_count == 0 && total_count > 0 {
            return Err(RepositoryError::Conflict(
                "Cannot delete membership with only inherited roles".to_string(),
            ));
        }

        // Delete member roles first (cascade)
        sqlx::query("DELETE FROM member_roles WHERE member_id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        // Delete member
        sqlx::query("DELETE FROM members WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

/// Utility functions for working with member with roles
impl MemberWithRoles {
    /// Get the member row
    pub fn member(&self) -> &MemberRow {
        &self.member
    }

    /// Get role IDs
    pub fn role_ids(&self) -> &[i64] {
        &self.role_ids
    }

    /// Check if member has a specific role
    pub fn has_role(&self, role_id: i64) -> bool {
        self.role_ids.contains(&role_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_member_with_roles() {
        let member = MemberRow {
            id: 1,
            user_id: 2,
            project_id: Some(3),
            entity_type: None,
            entity_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let member_with_roles = MemberWithRoles {
            member,
            role_ids: vec![1, 2, 3],
        };

        assert!(member_with_roles.has_role(1));
        assert!(member_with_roles.has_role(2));
        assert!(!member_with_roles.has_role(99));
    }
}
