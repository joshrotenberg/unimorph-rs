//! Export functionality for UniMorph data.
//!
//! This module provides export capabilities to various formats, primarily
//! Parquet for efficient storage and ML pipeline integration.
//!
//! # Example
//!
//! ```ignore
//! use unimorph_core::Store;
//!
//! let store = Store::open("datasets.db")?;
//! store.export_parquet("ita", "italian.parquet")?;
//! ```

use std::path::Path;
use tracing::{debug, instrument};

#[cfg(test)]
use crate::Entry;
use crate::{Result, Store};

#[cfg(feature = "parquet")]
mod parquet_export {
    use super::*;
    use arrow::array::{ArrayRef, StringBuilder};
    use arrow::datatypes::{DataType, Field, Schema};
    use arrow::record_batch::RecordBatch;
    use parquet::arrow::ArrowWriter;
    use parquet::basic::Compression;
    use parquet::file::properties::WriterProperties;
    use std::fs::File;
    use std::sync::Arc;

    /// Export options for Parquet files.
    #[derive(Debug, Clone)]
    pub struct ParquetExportOptions {
        /// Compression algorithm to use (default: Snappy).
        pub compression: Compression,
        /// Batch size for writing (default: 10000).
        pub batch_size: usize,
    }

    impl Default for ParquetExportOptions {
        fn default() -> Self {
            Self {
                compression: Compression::SNAPPY,
                batch_size: 10_000,
            }
        }
    }

    impl Store {
        /// Export a language dataset to Parquet format.
        ///
        /// Parquet provides excellent compression (50-200x smaller than SQLite)
        /// and is ideal for distribution, archival, or ML pipeline integration.
        ///
        /// # Example
        ///
        /// ```ignore
        /// let store = Store::open("datasets.db")?;
        /// store.export_parquet("ita", "italian.parquet")?;
        /// ```
        #[instrument(level = "debug", skip(self, path), fields(lang = %lang))]
        pub fn export_parquet<P: AsRef<Path>>(&self, lang: &str, path: P) -> Result<usize> {
            debug!(path = %path.as_ref().display(), "exporting to parquet");
            self.export_parquet_with_options(lang, path, ParquetExportOptions::default())
        }

        /// Export a language dataset to Parquet with custom options.
        ///
        /// # Example
        ///
        /// ```ignore
        /// use parquet::basic::Compression;
        ///
        /// let options = ParquetExportOptions {
        ///     compression: Compression::ZSTD(Default::default()),
        ///     batch_size: 50_000,
        /// };
        /// store.export_parquet_with_options("ita", "italian.parquet", options)?;
        /// ```
        #[instrument(level = "debug", skip(self, path, options), fields(lang = %lang))]
        pub fn export_parquet_with_options<P: AsRef<Path>>(
            &self,
            lang: &str,
            path: P,
            options: ParquetExportOptions,
        ) -> Result<usize> {
            debug!(
                path = %path.as_ref().display(),
                compression = ?options.compression,
                batch_size = options.batch_size,
                "exporting to parquet with options"
            );
            // Define schema
            let schema = Arc::new(Schema::new(vec![
                Field::new("lemma", DataType::Utf8, false),
                Field::new("form", DataType::Utf8, false),
                Field::new("features", DataType::Utf8, false),
            ]));

            // Create writer
            let file = File::create(path.as_ref())?;
            let props = WriterProperties::builder()
                .set_compression(options.compression)
                .build();
            let mut writer =
                ArrowWriter::try_new(file, schema.clone(), Some(props)).map_err(|e| {
                    crate::Error::Database(rusqlite::Error::ToSqlConversionFailure(Box::new(
                        std::io::Error::other(e.to_string()),
                    )))
                })?;

            // Query all entries for the language
            let mut stmt = self
                .conn
                .prepare("SELECT lemma, form, features FROM entries WHERE lang = ?")?;

            let mut rows = stmt.query([lang])?;

            let mut lemmas = StringBuilder::new();
            let mut forms = StringBuilder::new();
            let mut features = StringBuilder::new();
            let mut total_count = 0;
            let mut batch_count = 0;

            while let Some(row) = rows.next()? {
                let lemma: String = row.get(0)?;
                let form: String = row.get(1)?;
                let feat: String = row.get(2)?;

                lemmas.append_value(&lemma);
                forms.append_value(&form);
                features.append_value(&feat);

                batch_count += 1;
                total_count += 1;

                // Write batch when full
                if batch_count >= options.batch_size {
                    let batch = RecordBatch::try_new(
                        schema.clone(),
                        vec![
                            Arc::new(lemmas.finish()) as ArrayRef,
                            Arc::new(forms.finish()) as ArrayRef,
                            Arc::new(features.finish()) as ArrayRef,
                        ],
                    )
                    .map_err(|e| {
                        crate::Error::Database(rusqlite::Error::ToSqlConversionFailure(Box::new(
                            std::io::Error::other(e.to_string()),
                        )))
                    })?;

                    writer.write(&batch).map_err(|e| {
                        crate::Error::Database(rusqlite::Error::ToSqlConversionFailure(Box::new(
                            std::io::Error::other(e.to_string()),
                        )))
                    })?;

                    lemmas = StringBuilder::new();
                    forms = StringBuilder::new();
                    features = StringBuilder::new();
                    batch_count = 0;
                }
            }

            // Write remaining entries
            if batch_count > 0 {
                let batch = RecordBatch::try_new(
                    schema.clone(),
                    vec![
                        Arc::new(lemmas.finish()) as ArrayRef,
                        Arc::new(forms.finish()) as ArrayRef,
                        Arc::new(features.finish()) as ArrayRef,
                    ],
                )
                .map_err(|e| {
                    crate::Error::Database(rusqlite::Error::ToSqlConversionFailure(Box::new(
                        std::io::Error::other(e.to_string()),
                    )))
                })?;

                writer.write(&batch).map_err(|e| {
                    crate::Error::Database(rusqlite::Error::ToSqlConversionFailure(Box::new(
                        std::io::Error::other(e.to_string()),
                    )))
                })?;
            }

            writer.close().map_err(|e| {
                crate::Error::Database(rusqlite::Error::ToSqlConversionFailure(Box::new(
                    std::io::Error::other(e.to_string()),
                )))
            })?;

            debug!(count = total_count, "parquet export complete");
            Ok(total_count)
        }
    }
}

#[cfg(feature = "parquet")]
pub use parquet_export::ParquetExportOptions;

impl Store {
    /// Export a language dataset to TSV format.
    ///
    /// TSV is the native UniMorph format and is useful for compatibility
    /// with other tools.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let store = Store::open("datasets.db")?;
    /// store.export_tsv("ita", "italian.tsv")?;
    /// ```
    #[instrument(level = "debug", skip(self, path), fields(lang = %lang))]
    pub fn export_tsv<P: AsRef<Path>>(&self, lang: &str, path: P) -> Result<usize> {
        use std::io::Write;
        debug!(path = %path.as_ref().display(), "exporting to TSV");

        let file = std::fs::File::create(path.as_ref())?;
        let mut writer = std::io::BufWriter::new(file);

        let mut stmt = self
            .conn
            .prepare("SELECT lemma, form, features FROM entries WHERE lang = ?")?;

        let mut rows = stmt.query([lang])?;
        let mut count = 0;

        while let Some(row) = rows.next()? {
            let lemma: String = row.get::<_, String>(0)?;
            let form: String = row.get::<_, String>(1)?;
            let features: String = row.get::<_, String>(2)?;

            writeln!(writer, "{}\t{}\t{}", lemma, form, features)?;
            count += 1;
        }

        writer.flush()?;
        debug!(count, "TSV export complete");
        Ok(count)
    }

    /// Export a language dataset to JSON Lines format.
    ///
    /// Each line is a JSON object with lemma, form, and features fields.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let store = Store::open("datasets.db")?;
    /// store.export_jsonl("ita", "italian.jsonl")?;
    /// ```
    #[instrument(level = "debug", skip(self, path), fields(lang = %lang))]
    pub fn export_jsonl<P: AsRef<Path>>(&self, lang: &str, path: P) -> Result<usize> {
        use std::io::Write;
        debug!(path = %path.as_ref().display(), "exporting to JSONL");

        let file = std::fs::File::create(path.as_ref())?;
        let mut writer = std::io::BufWriter::new(file);

        let mut stmt = self.conn.prepare(
            "SELECT lemma, form, features FROM entries WHERE lang = ? ORDER BY lemma, form",
        )?;

        let mut rows = stmt.query([lang])?;
        let mut count = 0;

        while let Some(row) = rows.next()? {
            let lemma: String = row.get::<_, String>(0)?;
            let form: String = row.get::<_, String>(1)?;
            let features: String = row.get::<_, String>(2)?;

            let entry = serde_json::json!({
                "lemma": lemma,
                "form": form,
                "features": features
            });

            writeln!(writer, "{}", entry)?;
            count += 1;
        }

        writer.flush()?;
        debug!(count, "JSONL export complete");
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::LangCode;
    use tempfile::TempDir;

    fn setup_store() -> (Store, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let mut store = Store::in_memory().unwrap();
        let lang: LangCode = "ita".parse().unwrap();

        let entries = vec![
            Entry::parse_line("parlare\tparlo\tV;IND;PRS;1;SG", 1).unwrap(),
            Entry::parse_line("parlare\tparli\tV;IND;PRS;2;SG", 2).unwrap(),
            Entry::parse_line("parlare\tparla\tV;IND;PRS;3;SG", 3).unwrap(),
            Entry::parse_line("essere\tsono\tV;IND;PRS;1;SG", 4).unwrap(),
            Entry::parse_line("essere\tsei\tV;IND;PRS;2;SG", 5).unwrap(),
        ];

        store.import(&lang, &entries, None).unwrap();
        (store, temp_dir)
    }

    #[test]
    fn export_tsv() {
        let (store, temp_dir) = setup_store();
        let path = temp_dir.path().join("test.tsv");

        let count = store.export_tsv("ita", &path).unwrap();
        assert_eq!(count, 5);

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("parlare\tparlo\tV;IND;PRS;1;SG"));
        assert!(content.contains("essere\tsono\tV;IND;PRS;1;SG"));
    }

    #[test]
    fn export_jsonl() {
        let (store, temp_dir) = setup_store();
        let path = temp_dir.path().join("test.jsonl");

        let count = store.export_jsonl("ita", &path).unwrap();
        assert_eq!(count, 5);

        let content = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 5);

        // Each line should be valid JSON
        for line in lines {
            let _: serde_json::Value = serde_json::from_str(line).unwrap();
        }
    }

    #[cfg(feature = "parquet")]
    #[test]
    fn export_parquet() {
        let (store, temp_dir) = setup_store();
        let path = temp_dir.path().join("test.parquet");

        let count = store.export_parquet("ita", &path).unwrap();
        assert_eq!(count, 5);

        // Verify file was created and has content
        let metadata = std::fs::metadata(&path).unwrap();
        assert!(metadata.len() > 0);
    }
}
