//! Access Control Module
//! 
//! Authentication, authorization, and session management

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};

use super::SecurityConfig;

/// Access control manager
pub struct AccessControl {
    config: SecurityConfig,
    sessions: Arc<RwLock<HashMap<String, Session>>>,
    users: Arc<RwLock<HashMap<String, User>>>,
    rate_limiter: Arc<RwLock<RateLimiter>>,
}

/// User account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub user_id: String,
    pub username: String,
    pub role: UserRole,
    pub permissions: Vec<Permission>,
    pub created_at: u64,
    pub last_login: Option<u64>,
    pub failed_login_attempts: u32,
    pub locked: bool,
}

/// User roles
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UserRole {
    Admin,
    Trader,
    Viewer,
    System,
}

/// Permissions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Permission {
    ExecuteTrades,
    ViewPositions,
    ModifyConfig,
    ViewAuditLogs,
    EmergencyStop,
    ManageUsers,
    ViewMetrics,
}

/// User session
#[derive(Debug, Clone)]
pub struct Session {
    pub session_id: String,
    pub user_id: String,
    pub created_at: u64,
    pub last_activity: u64,
    pub ip_address: String,
    pub user_agent: Option<String>,
    pub permissions: Vec<Permission>,
}

/// Rate limiter
#[derive(Debug, Clone)]
pub struct RateLimiter {
    pub requests: HashMap<String, Vec<u64>>, // IP -> timestamps
    pub limit_per_minute: u32,
}

impl AccessControl {
    /// Create new access control manager
    pub fn new(config: &SecurityConfig) -> Result<Self> {
        info!("🔐 Initializing Access Control...");
        
        let sessions = Arc::new(RwLock::new(HashMap::new()));
        let users = Arc::new(RwLock::new(HashMap::<String, User>::new()));
        let rate_limiter = Arc::new(RwLock::new(RateLimiter::new(config.api_rate_limit_per_minute)));
        
        // Create default admin user
        let mut users_map = HashMap::new();
        users_map.insert("admin".to_string(), User::create_admin());
        
        info!("✅ Access Control initialized");
        
        Ok(Self {
            config: config.clone(),
            sessions,
            users: Arc::new(RwLock::new(users_map)),
            rate_limiter,
        })
    }
    
    /// Authenticate user and create session
    pub async fn authenticate(&self, username: &str, password: &str, ip_address: &str) -> Result<Option<String>> {
        // Check rate limiting
        if !self.check_rate_limit(ip_address).await? {
            warn!("🚨 Rate limit exceeded for IP: {}", ip_address);
            return Ok(None);
        }
        
        let mut users = self.users.write().await;
        
        if let Some(user) = users.get_mut(username) {
            if user.locked {
                warn!("🔒 User {} is locked", username);
                return Ok(None);
            }
            
            // Simplified password check - in production use proper hashing
            if self.verify_password(password, username) {
                // Reset failed attempts
                user.failed_login_attempts = 0;
                user.last_login = Some(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs());
                
                // Create session
                let session_id = self.create_session(user, ip_address).await?;
                
                info!("✅ User {} authenticated successfully", username);
                Ok(Some(session_id))
            } else {
                // Increment failed attempts
                user.failed_login_attempts += 1;
                
                if user.failed_login_attempts >= 3 {
                    user.locked = true;
                    error!("🚨 User {} locked due to too many failed attempts", username);
                }
                
                warn!("❌ Authentication failed for user {}", username);
                Ok(None)
            }
        } else {
            warn!("❌ User {} not found", username);
            Ok(None)
        }
    }
    
    /// Validate session and check permissions
    pub async fn validate_session(&self, session_id: &str, required_permission: Permission) -> Result<bool> {
        let sessions = self.sessions.read().await;
        
        if let Some(session) = sessions.get(session_id) {
            // Check session timeout
            let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
            let timeout_seconds = self.config.session_timeout_minutes as u64 * 60;
            
            if now - session.last_activity > timeout_seconds {
                warn!("⏰ Session {} expired", session_id);
                return Ok(false);
            }
            
            // Check permission
            if session.permissions.contains(&required_permission) {
                debug!("✅ Session {} has required permission {:?}", session_id, required_permission);
                return Ok(true);
            } else {
                warn!("🚫 Session {} lacks permission {:?}", session_id, required_permission);
                return Ok(false);
            }
        }
        
        warn!("❌ Invalid session: {}", session_id);
        Ok(false)
    }
    
    /// Update session activity
    pub async fn update_session_activity(&self, session_id: &str) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        
        if let Some(session) = sessions.get_mut(session_id) {
            session.last_activity = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        }
        
        Ok(())
    }
    
    /// Logout and invalidate session
    pub async fn logout(&self, session_id: &str) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        
        if sessions.remove(session_id).is_some() {
            info!("👋 Session {} logged out", session_id);
        }
        
        Ok(())
    }
    
    /// Check rate limiting
    async fn check_rate_limit(&self, ip_address: &str) -> Result<bool> {
        let mut rate_limiter = self.rate_limiter.write().await;
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        let limit = rate_limiter.limit_per_minute;

        // Clean old requests (older than 1 minute)
        let requests = rate_limiter.requests.entry(ip_address.to_string()).or_insert_with(Vec::new);
        requests.retain(|&timestamp| now - timestamp < 60);

        // Check if under limit
        if requests.len() < limit as usize {
            requests.push(now);
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    /// Create new session
    async fn create_session(&self, user: &User, ip_address: &str) -> Result<String> {
        let session_id = format!("sess_{}", uuid::Uuid::new_v4());
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        
        let session = Session {
            session_id: session_id.clone(),
            user_id: user.user_id.clone(),
            created_at: now,
            last_activity: now,
            ip_address: ip_address.to_string(),
            user_agent: None,
            permissions: user.permissions.clone(),
        };
        
        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id.clone(), session);
        
        Ok(session_id)
    }
    
    /// Verify password (simplified)
    fn verify_password(&self, password: &str, username: &str) -> bool {
        // In production, use proper password hashing (bcrypt, argon2, etc.)
        // This is a simplified check for demo purposes
        match username {
            "admin" => password == "admin123",
            _ => false,
        }
    }
    
    /// Get access control statistics
    pub async fn get_access_stats(&self) -> AccessStats {
        let sessions = self.sessions.read().await;
        let users = self.users.read().await;
        
        AccessStats {
            active_sessions: sessions.len() as u32,
            total_users: users.len() as u32,
            locked_users: users.values().filter(|u| u.locked).count() as u32,
            admin_users: users.values().filter(|u| u.role == UserRole::Admin).count() as u32,
        }
    }
}

impl User {
    /// Create default admin user
    fn create_admin() -> Self {
        Self {
            user_id: "admin".to_string(),
            username: "admin".to_string(),
            role: UserRole::Admin,
            permissions: vec![
                Permission::ExecuteTrades,
                Permission::ViewPositions,
                Permission::ModifyConfig,
                Permission::ViewAuditLogs,
                Permission::EmergencyStop,
                Permission::ManageUsers,
                Permission::ViewMetrics,
            ],
            created_at: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
            last_login: None,
            failed_login_attempts: 0,
            locked: false,
        }
    }
}

impl RateLimiter {
    fn new(limit_per_minute: u32) -> Self {
        Self {
            requests: HashMap::new(),
            limit_per_minute,
        }
    }
}

/// Access control statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessStats {
    pub active_sessions: u32,
    pub total_users: u32,
    pub locked_users: u32,
    pub admin_users: u32,
}
