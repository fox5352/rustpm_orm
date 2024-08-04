pub mod database {
    use serde_derive::{Deserialize, Serialize};
    use sled::{Db, open};

    //  ----------------------------------------------------------------
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

    pub struct ImageDatabase {
        conn: Db,
        pub _database_name: String
    }

    impl ImageDatabase {
        pub fn new(database_name: String) -> ImageDatabase {
            let name = database_name.clone();
            let path = std::path::Path::new(&database_name);
            // let conn = open(database_name).unwrap();
            let conn = open(path).unwrap();

            return ImageDatabase {
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

    impl Drop for ImageDatabase {
        fn drop(&mut self) {
            self.close()
        }
    }

    //  ----------------------------------------------------------------
    // pub struct BibleBookData {
    //     conn: Db,
    //     pub _database_name: String,
    // }


    //  ----------------------------------------------------------------
    #[derive(Debug, Deserialize, Serialize)]
    pub struct BibleVerse {
        pub verse_id: u32,
        pub book_name: String,
        pub book_number: u32,
        pub chapter: u32,
        pub verse: u32,
        pub text: String,
    }
    
    pub struct BibleVerseData {
        pub conn: Db,
        pub _database_name: String,
    }

    impl BibleVerseData {
        pub fn new(database_name: String) -> BibleVerseData {
            let name = database_name.clone();
            let path = std::path::Path::new(&database_name);
            // let conn = open(database_name).unwrap();
            let conn = open(path).unwrap();

            return BibleVerseData {
                conn,
                _database_name: name.to_owned()
            };
        }

        pub fn insert_bible_verse(&self, verse: BibleVerse) -> Result<String, std::io::Error> {
            let serialized_verse = match bincode::serialize(&verse) {
                Err(_) => return Err(std::io::Error::new(std::io::ErrorKind::Other, "failed to serialize verse data")),
                Ok(data) => data,
            };

            match self.conn.insert(verse.verse_id.to_be_bytes(), serialized_verse) {
                Err(_) => return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "failed to serialize verse data")),
                Ok(_) => (),
            }

            return Ok("successfully inserted verse data".to_string());
        }

        pub fn get_verse_by_id(&self, id: u32) -> Option<BibleVerse> {
            self.conn.get(id.to_be_bytes()).unwrap().and_then(|ivec| bincode::deserialize(&ivec).ok())
        }

        pub fn get_all_bible_verses(&self) -> Vec<BibleVerse> {
            self.conn.iter().filter_map(|result| {
                result.ok().and_then(|(_, v)| bincode::deserialize(&v).ok())
            }).collect()
        }

        pub fn get_verses_by_list_of_ids(&self, ids: Vec<u32>) -> Vec<BibleVerse> {
            let filterd_data: Vec<BibleVerse> = ids.iter().map(|id| self.get_verse_by_id(*id)).into_iter().filter(|verse| verse.is_some()).map(|verse| verse.unwrap()).collect();
            return filterd_data;
        }

        // TODO: DELETE method
        pub fn delte_by_id(&self, id: u32) -> Result<String, std::io::Error> {
            match self.conn.remove(id.to_be_bytes()) {
                Ok(Some(_)) => Ok("Successfully removed from database".to_string()),
                Ok(None) => Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Verse not found")),
                Err(_) => Err(std::io::Error::new(std::io::ErrorKind::Other, "Failed to remove verse from database")),
            }
        }

        pub fn close(&self) {
            self.conn.flush().unwrap();
        }
    }

    impl Drop for BibleVerseData {
        fn drop(&mut self) {
            self.close();
        }
    }

}


pub mod utils {
    use serde_derive::{Deserialize, Serialize};
    use std::fs;
    
    use super::database::BibleVerse;

    #[derive(Debug, Deserialize, Serialize)]
    pub struct BibleBook {
        pub book_number: u32,
        pub book_name: String,
        pub bible_verse_ids: Vec<u32>,
    }

    pub fn read_bible_csv(file_path: &str) -> Result<Vec<BibleVerse>, std::io::Error> {
        let contents = fs::read_to_string(file_path)?;
        let mut rows: Vec<BibleVerse> = Vec::new();

        let mut lines = contents.lines();

        lines.next().unwrap(); // ead useless line

        for line in lines {
            let fields: Vec<&str> = line.split(",").collect();
            if fields.len() == 6 {
                let verse_id: u32 = fields[0].parse().unwrap();
                let book_name = fields[1].to_string();
                let book_number: u32 = fields[2].parse().unwrap();
                let chapter: u32 = fields[3].parse().unwrap();
                let verse: u32 = fields[4].parse().unwrap();
                let text = fields[5].to_string();

                rows.push(BibleVerse {
                    verse_id,
                    book_name,
                    book_number,
                    chapter,
                    verse,
                    text,
                });
            }
        }

        Ok(rows)
    }

}

#[cfg(test)]
mod tests {
    use crate::database::BibleVerse;

    use super::database;
    use std::sync::Mutex;
    use lazy_static::lazy_static;

    // ----------------------------------------------------------------
    lazy_static! {
        static ref DB_MANAGER: Mutex<database::ImageDatabase> = Mutex::new(
            database::ImageDatabase::new("test_images".to_string())
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

    // ----------------------------------------------------------------
    lazy_static! {
        static ref BVD_MANAGER: Mutex<database::BibleVerseData> = Mutex::new(
            database::BibleVerseData::new("test_verses".to_string())
        );
    }

    use super::utils::read_bible_csv;

    #[test]
    fn test_insert_verse_data() {
        let db = BVD_MANAGER.lock().unwrap();

        let data = read_bible_csv("./test.csv").unwrap();

        let test_data = BibleVerse {
            verse_id: data[0].verse_id,
            book_name: data[0].book_name.clone(),
            book_number: data[0].book_number,
            chapter: data[0].chapter,
            verse: data[0].verse,
            text: data[0].text.clone(),
        };
        
        assert!(db.insert_bible_verse(test_data).is_ok());
    }

    #[test]
    fn test_get_all_verses() {
        let db = BVD_MANAGER.lock().unwrap();

        let data = read_bible_csv("./test.csv").unwrap();

        for block in data.into_iter() {
            db.insert_bible_verse(block).unwrap();
        }

        let verses = db.get_all_bible_verses();
        
        assert!(!verses.is_empty());
    }

    #[test]
    fn test_get_verse() {
        let db = BVD_MANAGER.lock().unwrap();
        
        let data = read_bible_csv("./test.csv").unwrap();

        for block in data.into_iter() {
            db.insert_bible_verse(block).unwrap();
        }

        let verses = db.get_all_bible_verses();

        let verse = verses.iter().next().unwrap();

        let match_verse = db.get_verse_by_id(verse.verse_id).unwrap();

        assert_eq!(verse.verse_id, match_verse.verse_id);
    }

    #[test]
    fn test_get_verses_by_list_of_ids() {
        let db = BVD_MANAGER.lock().unwrap();
        
        let data = read_bible_csv("./test.csv").unwrap();

        for block in data.into_iter() {
            db.insert_bible_verse(block).unwrap();
        }

        let verses = db.get_all_bible_verses();

        let ids = verses.iter().map(|v| v.verse_id).take(3).collect::<Vec<u32>>();

        let match_ids: Vec<u32> = db.get_verses_by_list_of_ids(ids.clone()).into_iter().map(|data| data.verse_id).collect();

        
        let mut pass_test = true;

        for (idx, _value) in ids.iter().enumerate() {
            if ids[idx] != match_ids[idx] {
                pass_test = false; 
            }
        }

        assert!(pass_test)
    }
}