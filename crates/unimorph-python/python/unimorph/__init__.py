"""UniMorph Python bindings - morphological data toolkit."""

from unimorph._internal import DatasetStats, Entry
from unimorph._internal import Store as _Store
from unimorph._internal import download

__all__ = ["Store", "Entry", "DatasetStats", "download"]


class Store(_Store):
    """UniMorph data store with Polars DataFrame support.

    Example:
        >>> from unimorph import Store, download
        >>> download("ita")  # Download Italian data
        >>> store = Store()
        >>> forms = store.inflect("ita", "parlare")
        >>> df = store.inflect_df("ita", "parlare")  # As Polars DataFrame
    """

    def inflect_df(self, lang: str, lemma: str):
        """Get inflected forms as a Polars DataFrame."""
        import polars as pl

        entries = self.inflect(lang, lemma)
        return pl.DataFrame(
            {
                "lemma": [e.lemma for e in entries],
                "form": [e.form for e in entries],
                "features": [e.features for e in entries],
            }
        )

    def analyze_df(self, lang: str, form: str):
        """Analyze a form and return results as a Polars DataFrame."""
        import polars as pl

        entries = self.analyze(lang, form)
        return pl.DataFrame(
            {
                "lemma": [e.lemma for e in entries],
                "form": [e.form for e in entries],
                "features": [e.features for e in entries],
            }
        )

    def search_features_df(self, lang: str, features: str, limit: int | None = None):
        """Search for entries with features and return as a Polars DataFrame."""
        import polars as pl

        entries = self.search_features(lang, features, limit)
        return pl.DataFrame(
            {
                "lemma": [e.lemma for e in entries],
                "form": [e.form for e in entries],
                "features": [e.features for e in entries],
            }
        )
