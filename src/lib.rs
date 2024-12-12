pub mod database {
    use serde::{Deserialize, Serialize};
    use sled::{open, Db};
    use uuid::Uuid;

    #[derive(Debug)]
    pub enum DBErrorKind {
        NotFound(String),
        WriteFailed(String),
        ReadFailed(String),
        Other(String)
    }

    #[derive(Debug)]
    pub struct DBError {
        kind: DBErrorKind,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    }

    impl DBError {
        pub fn new(kind:DBErrorKind) -> Self {
            return DBError { kind, source: Option::None }
        }

        pub fn with_source(kind: DBErrorKind, source: impl std::error::Error + Send + Sync + 'static) -> Self {
            return DBError {
                kind,
                source: Some(Box::new(source)),
            }
        }

        pub fn kind(&self) -> &DBErrorKind {
            return &self.kind
        }
    }

    impl std::fmt::Display for DBError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            return match &self.kind {
                DBErrorKind::NotFound(msg) => write!(f, "NotFound {}",msg),
                DBErrorKind::ReadFailed(msg) => write!(f, "failed to read from database {}",msg),
                DBErrorKind::WriteFailed(msg) => write!(f, "failed to write to database {}", msg),
                DBErrorKind::Other(msg) => write!(f, "{}", msg)
            }
        }
    }

    impl std::error::Error for DBError {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            self.source.as_ref().map(|e| e.as_ref() as &(dyn std::error::Error + 'static))
        }
    }

    impl From<sled::Error> for DBError {
        fn from(err: sled::Error) -> Self {
            let kind = match err.clone() {
                sled::Error::CollectionNotFound(_) => DBErrorKind::NotFound("collection not found".to_string()),
                sled::Error::Io(msg) => DBErrorKind::Other(msg.to_string()),
                sled::Error::Unsupported(msg) => DBErrorKind::Other(msg),
                sled::Error::ReportableBug(_) => DBErrorKind::Other("Dependency failed in some way".to_string()),
                _ => DBErrorKind::Other("A corruption occurred".to_string())
            };

            return DBError::with_source(kind, err);
        }
    }

    pub fn gen_id() -> String {
        return Uuid::new_v4().to_string();
    }

    pub trait Id {
        fn gen_id(&self) -> String;
    }

    #[derive(Debug, Clone)]
    pub struct DBManager {
        conn: Db,
        pub database_name: String,
    }

    impl DBManager {
        pub fn gen_id(&self) -> String {
            return gen_id();
        }

        pub fn new(database_name: String) -> Result<DBManager, DBError> {
            let name = database_name.clone();
            let path = std::path::Path::new(&database_name);
            let conn = open(path)?;
            return Ok(DBManager {
                conn,
                database_name: name.to_owned(),
            });
        }

        pub fn insert_data<'a, T: Id + Deserialize<'a> + Serialize>(
            &self,
            data: T,
        ) -> Result<String, DBError>
        where
            T: Deserialize<'a> + Serialize + Id,
        {
            let serialized_data = match bincode::serialize(&data) {
                Err(_) => {
                    return Err(DBError::new(DBErrorKind::Other("failed to serialize data".to_string())))
                }
                Ok(data) => data,
            };

            let id = data.gen_id();

            match self.conn.insert(id.clone(), serialized_data) {
                Err(_) => {
                    return Err(DBError::new(DBErrorKind::Other("failed to serialize data".to_string())))
                }
                Ok(_) => Ok(id),
            }
        }

        pub fn get_by_id<T>(&self, id: String) -> Result<T, DBError>
        where
            T: for<'a> Deserialize<'a> + Serialize + Id,
        {
            let result = self.conn.get(id)?;
            if let Some(data) =  result.and_then(|ivec| bincode::deserialize(&ivec).ok()){
                return Ok(data);
            }else {
                return Err(DBError::new(DBErrorKind::ReadFailed("".to_string())));
            }
        }

        // TODO:: redo later
        // pub fn get_all_data<'b, T>(&self) -> Vec<T>
        // where
        //     T: for<'a> Deserialize<'a> + Serialize + Id,
        // {
        //     self.conn
        //         .iter()
        //         .filter_map(|result| result.ok().and_then(|(_, v)| bincode::deserialize(&v).ok()))
        //         .collect()
        // }

        pub fn delete_by_id(&self, id: String) -> Result<String, DBError> {
            if self.conn.get(id.clone()).is_ok() {
                if self.conn.remove(id)?.is_some() {
                    return  Ok("data successfully removed".to_string());
                }else {
                    return Err(DBError::new(DBErrorKind::ReadFailed("".to_string())));
                }
                
            }else {
                return Err(DBError::new(DBErrorKind::NotFound("delete operation failed".to_string())));
            }
        }

        pub fn close(&self) {
            self.conn.flush().unwrap();
        }
    }

    impl Drop for DBManager {
        fn drop(&mut self) {
            self.close()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::database::*;
    use serde_derive::{Deserialize, Serialize};
    use std::fs;

    // Helper test struct
    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestUser {
        id: String,
        name: String,
        age: u32,
    }

    impl Id for TestUser {
        fn gen_id(&self) -> String {
            return gen_id();
        }
    }

    // Helper function to clean up test database
    fn cleanup_test_db(db_name: &str) {
        let _ = fs::remove_dir_all(db_name);
    }

    #[test]
    fn test_db_creation() {
        let db_name = "test_create_db";
        cleanup_test_db(db_name);

        let db = DBManager::new(db_name.to_string());
        assert!(db.is_ok());

        cleanup_test_db(db_name);
    }

    #[test]
    fn test_gen_id() {
        let db_name = "test_gen_id_db";
        cleanup_test_db(db_name);

        let db = DBManager::new(db_name.to_string()).unwrap();
        let id1 = db.gen_id();
        let id2 = db.gen_id();

        assert_ne!(id1, id2, "Generated IDs should be unique");
        assert_eq!(id1.len(), 36, "UUID should be 36 characters long");

        cleanup_test_db(db_name);
    }

    #[test]
    fn test_insert_and_get_data() {
        let db_name = "test_insert_db";
        cleanup_test_db(db_name);

        let db = DBManager::new(db_name.to_string()).unwrap();
        let test_user = TestUser {
            id: db.gen_id(),
            name: "John Doe".to_string(),
            age: 30,
        };

        let inserted_id = db.insert_data(test_user.clone()).unwrap();
        let retrieved_user: TestUser = db.get_by_id(inserted_id.clone()).unwrap();

        assert_eq!(retrieved_user.id, test_user.id);
        assert_eq!(retrieved_user.name, test_user.name);
        assert_eq!(retrieved_user.age, test_user.age);

        cleanup_test_db(db_name);
    }

    #[test]
    fn test_get_nonexistent_data() {
        let db_name = "test_get_none_db";
        cleanup_test_db(db_name);

        let db = DBManager::new(db_name.to_string()).unwrap();
        let retrieved_user: Result<TestUser, DBError> = db.get_by_id("nonexistent_id".to_string());

        assert!(retrieved_user.is_err());

        cleanup_test_db(db_name);
    }

    #[test]
    fn test_delete_data() {
        let db_name = "test_delete_db";
        cleanup_test_db(db_name);

        let db = DBManager::new(db_name.to_string()).unwrap();
        let test_user = TestUser {
            id: db.gen_id(),
            name: "Jane Doe".to_string(),
            age: 25,
        };

        let inserted_id = db.insert_data(test_user).unwrap();
        let delete_result = db.delete_by_id(inserted_id.clone());
        assert!(delete_result.is_ok());

        let retrieved_user: Result<TestUser, DBError> = db.get_by_id(inserted_id);
        assert!(retrieved_user.is_err());

        cleanup_test_db(db_name);
    }

    #[test]
    fn test_delete_nonexistent_data() {
        let db_name = "test_delete_none_db";
        cleanup_test_db(db_name);

        let db = DBManager::new(db_name.to_string()).unwrap();
        let delete_result = db.delete_by_id("nonexistent_id".to_string());

        assert!(delete_result.is_err());

        cleanup_test_db(db_name);
    }

    #[test]
    fn test_database_close() {
        let db_name = "test_close_db";
        cleanup_test_db(db_name);

        {
            let db = DBManager::new(db_name.to_string()).unwrap();
            let test_user = TestUser {
                id: db.gen_id(),
                name: "Test User".to_string(),
                age: 20,
            };
            db.insert_data(test_user).unwrap();
            // Database will be automatically closed here due to Drop trait
        }

        // Verify we can open and read from the database again
        let db = DBManager::new(db_name.to_string()).unwrap();
        assert_eq!(db.database_name, db_name);

        cleanup_test_db(db_name);
    }
}
