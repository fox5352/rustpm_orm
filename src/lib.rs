pub mod database {
    use rusqlite::Connection;
    
    pub struct ImageData {
        pub name: String,
        pub data: Vec<u8>,
        pub file_type: String,
    }

    pub struct ImageDataDB {
        pub id: i32,
        pub title: String,
        pub data: Vec<u8>,
        pub file_type: String,
    }

    pub struct DBManager {
        pub conn: Connection,
        pub _database_name: String
    }

    impl DBManager {
        pub fn new(database_name: String) -> DBManager {
            let name = database_name.clone();

            let conn = Connection::open(database_name).unwrap();

            return DBManager {
                conn,
                _database_name: name.to_owned()
            };
        }

        // IMAGE TABEL MANAGEMENT FUNCTIONS
        pub fn create_image_table(&self) -> Result<(), rusqlite::Error>{
            let query = "CREATE TABLE IF NOT EXISTS images ( id INTEGER PRIMARY KEY, title VARCHAR(255) NOT NULL, data BLOB NOT NULL, type VARCHAR(50) NOT NULL)";

            return match self.conn.execute(query, []) {
                Err(e) => Err(e),
                Ok(_) => Ok(())
            };
        }

        pub fn insert_image(&self, image: ImageData) -> Result<String, rusqlite::Error> {
            let query = "INSERT INTO images (title, data, type) VALUES (?, ?, ?)";

            return match self.conn.execute(&query, (image.name.clone(), image.data.clone(), image.file_type.clone())) {
                Ok(_) => Ok(String::from("successfully inserted image")),
                Err(e) => Err(e)
            };
        }
    
        pub fn get_images(&self) -> Result<Vec<ImageDataDB>, rusqlite::Error> {
            let mut images: Vec<ImageDataDB> = Vec::new();
            
            let mut stmt = self.conn.prepare("SELECT id, title, data, type FROM images")?;
            let image_iter = stmt.query_map([], |row| {
                return Ok(ImageDataDB {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    data: row.get(2)?,
                    file_type: row.get(3)?,
                })
            })?;

            for image in image_iter {
                if let Ok(data) = image {
                    images.push(data)
                }
            }

            return Ok(images);
        }
    
        pub fn delete_image(&self, id: i32) -> Result<usize, rusqlite::Error> {
            let query = format!("DELETE FROM images WHERE id = {}", id);

            let res = self.conn.execute(&query, ())?;

            return Ok(res);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::database;

    #[test]
    fn test_create_image_table() {
        let db_manager = database::DBManager::new("test_images.db".to_string());
        assert!(db_manager.create_image_table().is_ok());
    }

    #[test]
    fn test_insert_image() {
        let test_image = database::ImageData {
            name: "Test Image".to_string(),
            data: vec![0, 1, 2, 3, 4, 5],
            file_type: "image/png".to_string(),
        };

        let db_manager = database::DBManager::new("test_images.db".to_string());
        assert!(db_manager.create_image_table().is_ok());
        
        assert!(db_manager.insert_image(test_image).is_ok());
    }

    #[test]
    fn test_get_images() {
        let db_manager = database::DBManager::new("test_images.db".to_string());
        assert!(db_manager.create_image_table().is_ok());

        let test_image = database::ImageData {
            name: "Test Image".to_string(),
            data: vec![0, 1, 2, 3, 4, 5],
            file_type: "image/png".to_string(),
        };

        db_manager.insert_image(test_image).unwrap();

        let images = db_manager.get_images().unwrap();
        assert!(images.len() > 0);
    }

    #[test]
    fn test_delete_image() {
        let db = database::DBManager::new("test_images.db".to_string());
        assert!(db.create_image_table().is_ok());

        let test_image = database::ImageData {
            name: "Test Image".to_string(),
            data: vec![0, 1, 2, 3, 4, 5],
            file_type: "png".to_string(),
        };
        
        assert!(db.insert_image(test_image).is_ok());


        let images = db.get_images().unwrap();

        let len_before = images.len();

        db.delete_image(images[0].id).unwrap();

        let images = db.get_images().unwrap();

        eprintln!("{} :: {}", images.len(), len_before);
    }
}
