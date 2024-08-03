pub mod database {
    use serde_derive::{Deserialize, Serialize};
    use sled::{Db, open};
    
    pub struct ImageData {
        pub name: String,
        pub data: Vec<u8>,
        pub file_type: String,
    }

    #[derive(Deserialize, Serialize, Debug)]
    pub struct ImageDataDB {
        pub id: i32,
        pub title: String,
        pub data: Vec<u8>,
        pub file_type: String,
    }

    pub struct DBManager {
        conn: Db,
        pub _database_name: String
    }

    impl DBManager {
        pub fn new(database_name: String) -> DBManager {
            let name = database_name.clone();
            let path = std::path::Path::new(&database_name);
            // let conn = open(database_name).unwrap();
            let conn = open(path).unwrap();

            return DBManager {
                conn,
                _database_name: name.to_owned()
            };
        }

        pub fn insert_image(&self, image: ImageData) -> Result<String, std::io::Error> {
            let id = match self.conn.generate_id() {
                Err(_) => return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Failed to generate ID")),   
                Ok(id) => id as i32,
            };

            let new_image = ImageDataDB {
                id: id,
                title: image.name,
                data: image.data,
                file_type: image.file_type,
            };

            let serialized_image = match bincode::serialize(&new_image) {
                Err(_) => return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "failed to serialize image data")),
                Ok(data) => data,
            };
            

            match self.conn.insert(id.to_be_bytes(), serialized_image) {
                Err(_) => return Err(std::io::Error::new(std::io::ErrorKind::Other, "failed to insert image data")),
                Ok(data) => data,
            };

            return Ok("successfully inserted image data".to_string());
        }
        
        pub fn get_image(&self, id: u64) -> Option<ImageDataDB> {
            self.conn.get(id.to_be_bytes()).unwrap().and_then(|ivec| bincode::deserialize(&ivec).ok())
        }
    
        pub fn get_images(&self) -> Vec<ImageDataDB> {
            self.conn.iter().filter_map(|result| {
                result.ok().and_then(|(_, v)| bincode::deserialize(&v).ok())
            }).collect()
        
        }
        
        pub fn delete_image(&self, id: i32) -> Result<String, std::io::Error> {
            match self.conn.remove(id.to_be_bytes()) {
                Ok(Some(_)) => Ok("Successfully removed image".to_string()),
                Ok(None) => Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Image not found")),
                Err(_) => Err(std::io::Error::new(std::io::ErrorKind::Other, "Failed to remove image")),
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
    use super::database;
    use std::sync::Mutex;
    use lazy_static::lazy_static;

    lazy_static! {
        static ref DB_MANAGER: Mutex<database::DBManager> = Mutex::new(
            database::DBManager::new("test_images".to_string())
        );
    }

    #[test]
    fn test_insert_image() {
        let test_image = database::ImageData {
            name: "Test Image".to_string(),
            data: vec![0, 1, 2, 3, 4, 5],
            file_type: "png".to_string(),
        };

        let db_manager = DB_MANAGER.lock().unwrap();
        assert!(db_manager.insert_image(test_image).is_ok());
    }

    #[test]
    fn test_get_images() {
        let test_image = database::ImageData {
            name: "Test Image".to_string(),
            data: vec![0, 1, 2, 3, 4, 5],
            file_type: "png".to_string(),
        };

        let db_manager = DB_MANAGER.lock().unwrap();
        assert!(db_manager.insert_image(test_image).is_ok());

        let images = db_manager.get_images();
        assert!(images.len() > 0);
    }

    #[test]
    fn test_delete_image() {
        let db_manager = DB_MANAGER.lock().unwrap();
        
        // Clear the database before the test
        for image in db_manager.get_images() {
            db_manager.delete_image(image.id).unwrap();
        }

        let test_image = database::ImageData {
            name: "Test Image".to_string(),
            data: vec![0, 1, 2, 3, 4, 5],
            file_type: "png".to_string(),
        };

        db_manager.insert_image(test_image).unwrap();

        let images = db_manager.get_images();
        assert_eq!(images.len(), 1);

        let image_to_delete = &images[0];
        let delete_result = db_manager.delete_image(image_to_delete.id);
        assert!(delete_result.is_ok());

        let images_after_delete = db_manager.get_images();
        assert_eq!(images_after_delete.len(), 0);
    }
}