pub mod database {
    use serde::{Deserialize, Serialize};
    use sled::{open, Db};
    use std::io::Error;

    /// A trait for types that have a unique identifier.
    pub trait Id {
        /// Returns the unique identifier for this instance.
        fn get_id(&self) -> u32;
    }

    /// Database manager that provides CRUD operations with automatic ID management.
    ///
    /// The `DBManager` wraps a key-value database and handles:
    /// * Automatic ID generation
    /// * Serialization/deserialization of data
    /// * Basic CRUD operations
    /// * Automatic resource cleanup
    ///
    /// # Example
    /// ```
    /// use your_crate::{DBManager, Id};
    ///
    /// #[derive(Debug, Serialize, Deserialize)]
    /// struct User {
    ///     id: u32,
    ///     name: String,
    /// }
    ///
    /// impl Id for User {
    ///     fn get_id(&self) -> u32 {
    ///         self.id
    ///     }
    /// }
    ///
    /// let db = DBManager::new("users.db".to_string());
    /// let user = User {
    ///     id: db.gen_id().unwrap(),
    ///     name: "John".to_string(),
    /// };
    /// db.insert_data(user).unwrap();
    /// ```
    #[derive(Debug, Clone)]
    pub struct DBManager {
        conn: Db,
        pub database_name: String,
    }

    impl DBManager {
        /// Generates a new unique identifier.
        ///
        /// # Errors
        ///
        /// Returns an error if the underlying database fails to generate an ID.
        pub fn gen_id(&self) -> Result<u32, Error> {
            match self.conn.generate_id() {
                Err(e) => {
                    eprintln!("Error {}", e);
                    return Err(Error::new(
                        std::io::ErrorKind::Other,
                        "failed to generate id".to_string(),
                    ));
                }
                Ok(id) => Ok(id as u32),
            }
        }

        /// Creates a new database manager instance.
        ///
        /// # Arguments
        ///
        /// * `database_name` - Path to the database file
        ///
        /// # Panics
        ///
        /// Panics if the database connection cannot be established.
        pub fn new(database_name: String) -> DBManager {
            let name = database_name.clone();
            let path = std::path::Path::new(&database_name);
            let conn = open(path).unwrap();
            return DBManager {
                conn,
                database_name: name.to_owned(),
            };
        }

        /// Inserts data into the database.
        ///
        /// The data must implement the `Id` trait to provide a unique identifier,
        /// and `Serialize`/`Deserialize` for storage.
        ///
        /// # Arguments
        ///
        /// * `data` - The data to insert
        ///
        /// # Errors
        ///
        /// Returns an error if:
        /// * Data serialization fails
        /// * Database insertion fails
        pub fn insert_data<'a, T: Id + Deserialize<'a> + Serialize>(
            &self,
            data: T,
        ) -> Result<String, std::io::Error>
        where
            T: Deserialize<'a> + Serialize + Id,
        {
            let serialized_data = match bincode::serialize(&data) {
                Err(_) => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "failed to serialize data",
                    ))
                }
                Ok(data) => data,
            };

            let id = data.get_id();

            match self.conn.insert(id.to_be_bytes(), serialized_data) {
                Err(_) => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "failed to serialize data",
                    ))
                }
                Ok(_) => Ok("successfully inserted data".to_string()),
            }
        }

        /// Retrieves data by its ID.
        ///
        /// # Arguments
        ///
        /// * `id` - The unique identifier of the record to retrieve
        ///
        /// # Type Parameters
        ///
        /// * `T` - The type of data to deserialize into. Must implement `Id`, `Serialize`, and `Deserialize`
        ///
        /// # Returns
        ///
        /// Returns `None` if no data exists for the given ID.
        pub fn get_by_id<T>(&self, id: u32) -> Option<T>
        where
            T: for<'a> Deserialize<'a> + Serialize + Id,
        {
            self.conn
                .get(id.to_be_bytes())
                .unwrap()
                .and_then(|ivec| bincode::deserialize(&ivec).ok())
        }

        /// Retrieves all records from the database.
        ///
        /// # Type Parameters
        ///
        /// * `T` - The type of data to deserialize into. Must implement `Id`, `Serialize`, and `Deserialize`
        pub fn get_all_data<'b, T>(&self) -> Vec<T>
        where
            T: for<'a> Deserialize<'a> + Serialize + Id,
        {
            self.conn
                .iter()
                .filter_map(|result| result.ok().and_then(|(_, v)| bincode::deserialize(&v).ok()))
                .collect()
        }

        /// Deletes a record by its ID.
        ///
        /// # Arguments
        ///
        /// * `id` - The unique identifier of the record to delete
        ///
        /// # Errors
        ///
        /// Returns an error if:
        /// * No data exists for the given ID
        /// * Database deletion fails
        pub fn delete_by_id(&self, id: u32) -> Result<String, std::io::Error> {
            match self.conn.remove(id.to_be_bytes()) {
                Err(_) => Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "No data found for this id",
                )),
                Ok(_) => Ok("Successfully deleted data".to_string()),
            }
        }

        /// Flushes any pending database operations.
        ///
        /// This is automatically called when the `DBManager` is dropped.
        pub fn close(&self) {
            self.conn.flush().unwrap();
        }
    }

    /// Implements automatic resource cleanup when `DBManager` is dropped.
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
    use lazy_static::lazy_static;
    use serde_derive::{Deserialize, Serialize};
    use std::sync::Mutex;

    use super::database::Id;

    // ----------------------------------------------------------------
    lazy_static! {
        static ref DBM: Mutex<database::DBManager> =
            Mutex::new(database::DBManager::new("test_database".to_string()));
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

    impl Id for BibleVerseType {
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

        let new_data = mock_data.iter().map(|data| BibleVerseType {
            verse_id: data.verse_id,
            book_name: data.book_name.clone(),
            book_number: data.book_number,
            chapter: data.chapter,
            verse: data.verse,
            text: data.text.clone(),
        });

        for data in new_data {
            assert!(db.insert_data(data).is_ok());
        }

        let all_data = db.get_all_data::<BibleVerseType>();
        assert!(all_data.len() >= mock_data.len())
    }

    #[test]
    fn test_get_by_id() {
        let db = DBM.lock().unwrap();

        let mock_data = read_bible_csv("./test.csv").unwrap();

        let new_data: Vec<BibleVerseType> = mock_data
            .iter()
            .map(|data| BibleVerseType {
                verse_id: data.verse_id,
                book_name: data.book_name.clone(),
                book_number: data.book_number,
                chapter: data.chapter,
                verse: data.verse,
                text: data.text.clone(),
            })
            .collect();

        for data in new_data {
            assert!(db.insert_data(data).is_ok());
        }

        let data = db.get_by_id::<BibleVerseType>(mock_data[0].verse_id);
        assert!(data.is_some());
    }

    #[test]
    fn test_delete_by_id() {
        let db = DBM.lock().unwrap();

        let mock_data = read_bible_csv("./test.csv").unwrap();

        let new_data: Vec<BibleVerseType> = mock_data
            .iter()
            .map(|data| BibleVerseType {
                verse_id: data.verse_id,
                book_name: data.book_name.clone(),
                book_number: data.book_number,
                chapter: data.chapter,
                verse: data.verse,
                text: data.text.clone(),
            })
            .collect();

        for data in new_data {
            assert!(db.insert_data(data).is_ok());
        }

        let before_len = db.get_all_data::<BibleVerseType>().len() as u64;

        db.delete_by_id(mock_data[0].verse_id).unwrap();

        let after_len = db.get_all_data::<BibleVerseType>().len() as u64;

        assert!(before_len > after_len);
    }
}
