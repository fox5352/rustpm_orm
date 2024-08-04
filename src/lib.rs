pub mod database {
    use std::io::Error;

    use serde::{Deserialize, Serialize};
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

    #[derive(Debug, Deserialize, Serialize)]
    pub struct BibleBook {
        pub book_number: u32,
        pub book_name: String,
        pub bible_verse_ids: Vec<u32>,
    }

    pub trait Id {
        fn get_id(&self) -> u32;
    }
    
    pub struct DBManager {
        conn: Db,
        _database_name: String,
    }
    
    impl DBManager {
        pub fn gen_id(&self) -> Result<u32, Error> {
            match self.conn.generate_id() {
                Err(e) => {
                    eprintln!("Error {}", e);
                    return Err(Error::new(std::io::ErrorKind::Other, "failed to generate id".to_string()));
                },
                Ok(id) => Ok(id as u32),
            }
        }

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

        pub fn insert_data<'a, T: Id+ Deserialize<'a>+ Serialize>(&self, data: T) -> Result<String, std::io::Error> 
            where T: Deserialize<'a>+ Serialize + Id {
            let serialized_data = match bincode::serialize(&data) {
                Err(_) => return Err(std::io::Error::new(std::io::ErrorKind::Other, "failed to serialize data")),
                Ok(data) => data,
            };

            let id = data.get_id();

            match self.conn.insert(id.to_be_bytes(), serialized_data) {
                Err(_) => return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "failed to serialize data")),
                Ok(_) => Ok("successfully inserted data".to_string()),
            }
        }

        pub fn get_by_id<T>(&self, id: u32) -> Option<T>
            where T: for<'a> Deserialize<'a> + Serialize + Id {
            self.conn
                .get(id.to_be_bytes())
                .unwrap()
                .and_then(|ivec| bincode::deserialize(&ivec).ok())
        }
        
        pub fn get_all_data<'b, T>(&self) -> Vec<T> 
            where T: for<'a> Deserialize<'a> + Serialize + Id {
            self.conn
                .iter()
                .filter_map(|result| {
                    result.ok().and_then(|(_, v)| bincode::deserialize(&v).ok())
                })
                .collect()
        }
    
        pub fn delete_by_id(&self, id: u32) -> Result<String, std::io::Error> {
            match self.conn.remove(id.to_be_bytes()) {
                Err(_) => Err(std::io::Error::new(std::io::ErrorKind::NotFound, "No data found for this id")),
                Ok(_) => Ok("Successfully deleted data".to_string()),
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


pub mod utils {
    use std::fs;

    pub struct BibleVerse {
        pub verse_id: u32,
        pub book_name: String,
        pub book_number: u32,
        pub chapter: u32,
        pub verse: u32,
        pub text: String,
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
    use crate::utils::read_bible_csv;

    use super::database;
    use std::sync::Mutex;
    use lazy_static::lazy_static;
    use serde_derive::{Deserialize, Serialize};

    use super::database::Id;

    // ----------------------------------------------------------------
    lazy_static! {
        static ref DBM: Mutex<database::DBManager> = Mutex::new(
            database::DBManager::new("test_database".to_string())
        );
    }


    // testing bible verse
    #[derive(Debug, Deserialize, Serialize)]
    struct BibleVerseType {
        verse_id: u32,
        book_name: String,
        book_number: u32,
        chapter: u32,
        verse: u32,
        text: String,
    }

    impl Id for BibleVerseType{
        fn get_id(&self) -> u32 {
            self.verse_id
        }
    }

    #[test]
    fn test_insert() {
        let db = DBM.lock().unwrap();

        let test_data = read_bible_csv("./test.csv").unwrap();

        let test_data = BibleVerseType {
            verse_id: test_data[0].verse_id,
            book_name: test_data[0].book_name.clone(),
            book_number: test_data[0].book_number,
            chapter: test_data[0].chapter,
            verse: test_data[0].verse,
            text: test_data[0].text.clone(),
        };
        
        assert!(db.insert_data(test_data).is_ok());
    }

    #[test]
    fn test_get_all_data() {
        let db = DBM.lock().unwrap();

        let mock_data = read_bible_csv("./test.csv").unwrap();
        
        let new_data = mock_data.iter().map(|data| {
            BibleVerseType {
                verse_id: data.verse_id,
                book_name: data.book_name.clone(),
                book_number: data.book_number,
                chapter: data.chapter,
                verse: data.verse,
                text: data.text.clone(),
            }
        });

        for data in new_data {
            assert!(db.insert_data(data).is_ok());
        }

        let all_data = db.get_all_data::<BibleVerseType>();
        assert!(all_data.len() >= mock_data.len())

    }
    
}