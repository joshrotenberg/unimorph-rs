//! Python bindings for UniMorph morphological data toolkit.

use std::sync::Mutex;

use pyo3::prelude::*;
use unimorph_core::{DatasetStats, Entry, Repository};

/// A morphological entry from the UniMorph dataset.
#[pyclass(name = "Entry")]
#[derive(Clone)]
pub struct PyEntry {
    #[pyo3(get)]
    pub lemma: String,
    #[pyo3(get)]
    pub form: String,
    #[pyo3(get)]
    pub features: String,
}

impl From<Entry> for PyEntry {
    fn from(entry: Entry) -> Self {
        Self {
            lemma: entry.lemma,
            form: entry.form,
            features: entry.features.to_string(),
        }
    }
}

#[pymethods]
impl PyEntry {
    fn __repr__(&self) -> String {
        format!(
            "Entry(lemma='{}', form='{}', features='{}')",
            self.lemma, self.form, self.features
        )
    }
}

/// Statistics about a downloaded language dataset.
#[pyclass(name = "DatasetStats")]
#[derive(Clone)]
pub struct PyDatasetStats {
    #[pyo3(get)]
    pub language: String,
    #[pyo3(get)]
    pub total_entries: usize,
    #[pyo3(get)]
    pub unique_lemmas: usize,
    #[pyo3(get)]
    pub unique_forms: usize,
    #[pyo3(get)]
    pub unique_features: usize,
}

impl From<DatasetStats> for PyDatasetStats {
    fn from(stats: DatasetStats) -> Self {
        Self {
            language: stats.language,
            total_entries: stats.total_entries,
            unique_lemmas: stats.unique_lemmas,
            unique_forms: stats.unique_forms,
            unique_features: stats.unique_features,
        }
    }
}

#[pymethods]
impl PyDatasetStats {
    fn __repr__(&self) -> String {
        format!(
            "DatasetStats(language='{}', total_entries={}, unique_lemmas={}, unique_forms={}, unique_features={})",
            self.language,
            self.total_entries,
            self.unique_lemmas,
            self.unique_forms,
            self.unique_features
        )
    }
}

/// The main store for UniMorph data.
#[pyclass(name = "Store")]
pub struct PyStore {
    repo: Mutex<Repository>,
}

#[pymethods]
impl PyStore {
    /// Create a new store with the default cache directory.
    #[new]
    fn new() -> PyResult<Self> {
        let repo = Repository::new()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
        Ok(Self {
            repo: Mutex::new(repo),
        })
    }

    /// Get all inflected forms for a lemma in a language.
    fn inflect(&self, lang: &str, lemma: &str) -> PyResult<Vec<PyEntry>> {
        let repo = self
            .repo
            .lock()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        let entries = repo
            .store()
            .inflect(lang, lemma)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
        Ok(entries.into_iter().map(PyEntry::from).collect())
    }

    /// Analyze a word form to find possible lemmas and features.
    fn analyze(&self, lang: &str, form: &str) -> PyResult<Vec<PyEntry>> {
        let repo = self
            .repo
            .lock()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        let entries = repo
            .store()
            .analyze(lang, form)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
        Ok(entries.into_iter().map(PyEntry::from).collect())
    }

    /// Get statistics for a language dataset.
    fn stats(&self, lang: &str) -> PyResult<Option<PyDatasetStats>> {
        let repo = self
            .repo
            .lock()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        let stats = repo
            .store()
            .stats(lang)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
        Ok(stats.map(PyDatasetStats::from))
    }

    /// List all downloaded languages.
    fn languages(&self) -> PyResult<Vec<String>> {
        let repo = self
            .repo
            .lock()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        let langs = repo
            .store()
            .languages()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
        Ok(langs)
    }

    /// Check if a language is downloaded.
    fn has_language(&self, lang: &str) -> PyResult<bool> {
        let repo = self
            .repo
            .lock()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        let has = repo
            .store()
            .has_language(lang)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
        Ok(has)
    }

    /// Search for entries containing specific features.
    fn search_features(
        &self,
        lang: &str,
        features: &str,
        limit: Option<usize>,
    ) -> PyResult<Vec<PyEntry>> {
        let repo = self
            .repo
            .lock()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        let entries = repo
            .store()
            .search_features(lang, features, limit)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
        Ok(entries.into_iter().map(PyEntry::from).collect())
    }
}

/// Download a language dataset.
#[pyfunction]
fn download(lang: &str) -> PyResult<()> {
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

    rt.block_on(async {
        let repo = Repository::new()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
        repo.download(lang)
            .await
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
        Ok(())
    })
}

/// Python module for UniMorph.
#[pymodule]
fn _internal(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyEntry>()?;
    m.add_class::<PyDatasetStats>()?;
    m.add_class::<PyStore>()?;
    m.add_function(wrap_pyfunction!(download, m)?)?;
    Ok(())
}
