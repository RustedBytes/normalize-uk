//! Optional Python bindings, enabled by the `python` Cargo feature.

#[pyo3::pymodule]
#[pyo3(name = "ukrainian_tn")]
mod bindings {
    use std::collections::HashMap;

    use pyo3::exceptions::{PyOSError, PyValueError};
    use pyo3::prelude::*;

    use crate::rozpodil;
    use crate::uktextnorm::{
        self, ColonStyle, CurrencySymbolPolicy, DateStyle, GrammaticalCase, InputTolerance,
        NormalizeOptions, NormalizePreset, NumericDateOrder, OrdinalForm, PhoneStyle, QuoteStyle,
        RangeStyle, SymbolStyle, UncertaintyCategory, UncertaintySeverity, VocabularyError,
    };

    fn invalid_choice(field: &str, value: &str, choices: &str) -> PyErr {
        PyValueError::new_err(format!("invalid {field} {value:?}; expected one of: {choices}"))
    }

    fn parse_preset(value: &str) -> PyResult<NormalizePreset> {
        match value {
            "default" => Ok(NormalizePreset::Default),
            "tts_friendly" => Ok(NormalizePreset::TtsFriendly),
            "conservative" => Ok(NormalizePreset::Conservative),
            "search_indexing" => Ok(NormalizePreset::SearchIndexing),
            _ => Err(invalid_choice(
                "preset",
                value,
                "default, tts_friendly, conservative, search_indexing",
            )),
        }
    }

    fn parse_range_style(value: &str) -> PyResult<RangeStyle> {
        match value {
            "compact" => Ok(RangeStyle::Compact),
            "from_to" => Ok(RangeStyle::FromTo),
            _ => Err(invalid_choice("range_style", value, "compact, from_to")),
        }
    }

    fn parse_phone_style(value: &str) -> PyResult<PhoneStyle> {
        match value {
            "grouped" => Ok(PhoneStyle::Grouped),
            "digit_by_digit" => Ok(PhoneStyle::DigitByDigit),
            _ => Err(invalid_choice("phone_style", value, "grouped, digit_by_digit")),
        }
    }

    fn parse_symbol_style(value: &str) -> PyResult<SymbolStyle> {
        match value {
            "expand" => Ok(SymbolStyle::Expand),
            "preserve" => Ok(SymbolStyle::Preserve),
            _ => Err(invalid_choice("symbol_style", value, "expand, preserve")),
        }
    }

    fn parse_date_style(value: &str) -> PyResult<DateStyle> {
        match value {
            "formal" => Ok(DateStyle::Formal),
            "spoken" => Ok(DateStyle::Spoken),
            _ => Err(invalid_choice("date_style", value, "formal, spoken")),
        }
    }

    fn parse_colon_style(value: &str) -> PyResult<ColonStyle> {
        match value {
            "contextual" => Ok(ColonStyle::Contextual),
            "clock" => Ok(ColonStyle::Clock),
            "ratio" => Ok(ColonStyle::Ratio),
            _ => Err(invalid_choice("colon_style", value, "contextual, clock, ratio")),
        }
    }

    fn parse_numeric_date_order(value: &str) -> PyResult<NumericDateOrder> {
        match value {
            "day_month_year" => Ok(NumericDateOrder::DayMonthYear),
            "month_day_year" => Ok(NumericDateOrder::MonthDayYear),
            "preserve_ambiguous" => Ok(NumericDateOrder::PreserveAmbiguous),
            _ => Err(invalid_choice(
                "numeric_date_order",
                value,
                "day_month_year, month_day_year, preserve_ambiguous",
            )),
        }
    }

    fn parse_currency_symbol_policy(value: &str) -> PyResult<CurrencySymbolPolicy> {
        match value {
            "assume_common" => Ok(CurrencySymbolPolicy::AssumeCommon),
            "preserve_ambiguous" => Ok(CurrencySymbolPolicy::PreserveAmbiguous),
            _ => Err(invalid_choice(
                "currency_symbol_policy",
                value,
                "assume_common, preserve_ambiguous",
            )),
        }
    }

    fn parse_quote_style(value: &str) -> PyResult<QuoteStyle> {
        match value {
            "keep" => Ok(QuoteStyle::Keep),
            "guillemets" => Ok(QuoteStyle::Guillemets),
            "straight" => Ok(QuoteStyle::Straight),
            "strip" => Ok(QuoteStyle::Strip),
            _ => Err(invalid_choice("quote_style", value, "keep, guillemets, straight, strip")),
        }
    }

    fn parse_input_tolerance(value: &str) -> PyResult<InputTolerance> {
        match value {
            "strict" => Ok(InputTolerance::Strict),
            "asr" => Ok(InputTolerance::Asr),
            _ => Err(invalid_choice("input_tolerance", value, "strict, asr")),
        }
    }

    fn parse_ordinal_form(value: &str) -> PyResult<OrdinalForm> {
        match value {
            "nom_m" => Ok(OrdinalForm::NomM),
            "nom_n" => Ok(OrdinalForm::NomN),
            "nom_f" => Ok(OrdinalForm::NomF),
            "nom_pl" => Ok(OrdinalForm::NomPl),
            "gen" => Ok(OrdinalForm::Gen),
            "dat" => Ok(OrdinalForm::Dat),
            "prep" => Ok(OrdinalForm::Prep),
            "loc" => Ok(OrdinalForm::Loc),
            "pl" => Ok(OrdinalForm::Pl),
            "loc_pl" => Ok(OrdinalForm::LocPl),
            "acc_f" => Ok(OrdinalForm::AccF),
            "gen_f" => Ok(OrdinalForm::GenF),
            "ins" => Ok(OrdinalForm::Ins),
            "ins_f" => Ok(OrdinalForm::InsF),
            "ins_pl" => Ok(OrdinalForm::InsPl),
            "loc_f" => Ok(OrdinalForm::LocF),
            _ => Err(invalid_choice(
                "ordinal form",
                value,
                "nom_m, nom_n, nom_f, nom_pl, gen, dat, prep, loc, pl, loc_pl, acc_f, gen_f, ins, ins_f, ins_pl, loc_f",
            )),
        }
    }

    fn parse_grammatical_case(value: &str) -> PyResult<GrammaticalCase> {
        match value {
            "genitive" => Ok(GrammaticalCase::Genitive),
            "dative" => Ok(GrammaticalCase::Dative),
            "instrumental" => Ok(GrammaticalCase::Instrumental),
            "prepositional" => Ok(GrammaticalCase::Prepositional),
            _ => Err(invalid_choice(
                "grammatical case",
                value,
                "genitive, dative, instrumental, prepositional",
            )),
        }
    }

    /// Configuration for `normalize_with` and `flag_uncertain_with`.
    #[pyclass(
        name = "NormalizeOptions",
        module = "ukrainian_tn",
        get_all,
        set_all,
        skip_from_py_object
    )]
    #[derive(Clone)]
    struct PyNormalizeOptions {
        expand_known_acronyms: bool,
        spell_unknown_acronyms: bool,
        normalize_english_words: bool,
        transliterate_latin: bool,
        repair_homoglyphs: bool,
        validate_dates: bool,
        parse_thousand_separators: bool,
        normalize_network_addresses: bool,
        quote_style: String,
        range_style: String,
        phone_style: String,
        symbol_style: String,
        date_style: String,
        colon_style: String,
        numeric_date_order: String,
        currency_symbol_policy: String,
        input_tolerance: String,
        asr_vocabulary: Vec<String>,
        vocabulary: HashMap<String, String>,
    }

    impl From<NormalizeOptions> for PyNormalizeOptions {
        fn from(options: NormalizeOptions) -> Self {
            Self {
                expand_known_acronyms: options.expand_known_acronyms,
                spell_unknown_acronyms: options.spell_unknown_acronyms,
                normalize_english_words: options.normalize_english_words,
                transliterate_latin: options.transliterate_latin,
                repair_homoglyphs: options.repair_homoglyphs,
                validate_dates: options.validate_dates,
                parse_thousand_separators: options.parse_thousand_separators,
                normalize_network_addresses: options.normalize_network_addresses,
                quote_style: match options.quote_style {
                    QuoteStyle::Keep => "keep",
                    QuoteStyle::Guillemets => "guillemets",
                    QuoteStyle::Straight => "straight",
                    QuoteStyle::Strip => "strip",
                }
                .to_owned(),
                range_style: match options.range_style {
                    RangeStyle::Compact => "compact",
                    RangeStyle::FromTo => "from_to",
                }
                .to_owned(),
                phone_style: match options.phone_style {
                    PhoneStyle::Grouped => "grouped",
                    PhoneStyle::DigitByDigit => "digit_by_digit",
                }
                .to_owned(),
                symbol_style: match options.symbol_style {
                    SymbolStyle::Expand => "expand",
                    SymbolStyle::Preserve => "preserve",
                }
                .to_owned(),
                date_style: match options.date_style {
                    DateStyle::Formal => "formal",
                    DateStyle::Spoken => "spoken",
                }
                .to_owned(),
                colon_style: match options.colon_style {
                    ColonStyle::Contextual => "contextual",
                    ColonStyle::Clock => "clock",
                    ColonStyle::Ratio => "ratio",
                }
                .to_owned(),
                numeric_date_order: match options.numeric_date_order {
                    NumericDateOrder::DayMonthYear => "day_month_year",
                    NumericDateOrder::MonthDayYear => "month_day_year",
                    NumericDateOrder::PreserveAmbiguous => "preserve_ambiguous",
                }
                .to_owned(),
                currency_symbol_policy: match options.currency_symbol_policy {
                    CurrencySymbolPolicy::AssumeCommon => "assume_common",
                    CurrencySymbolPolicy::PreserveAmbiguous => "preserve_ambiguous",
                }
                .to_owned(),
                input_tolerance: match options.input_tolerance {
                    InputTolerance::Strict => "strict",
                    InputTolerance::Asr => "asr",
                }
                .to_owned(),
                asr_vocabulary: options.asr_vocabulary,
                vocabulary: options.vocabulary,
            }
        }
    }

    impl TryFrom<&PyNormalizeOptions> for NormalizeOptions {
        type Error = PyErr;

        fn try_from(options: &PyNormalizeOptions) -> PyResult<Self> {
            Ok(Self {
                expand_known_acronyms: options.expand_known_acronyms,
                spell_unknown_acronyms: options.spell_unknown_acronyms,
                normalize_english_words: options.normalize_english_words,
                transliterate_latin: options.transliterate_latin,
                repair_homoglyphs: options.repair_homoglyphs,
                validate_dates: options.validate_dates,
                parse_thousand_separators: options.parse_thousand_separators,
                normalize_network_addresses: options.normalize_network_addresses,
                quote_style: parse_quote_style(&options.quote_style)?,
                range_style: parse_range_style(&options.range_style)?,
                phone_style: parse_phone_style(&options.phone_style)?,
                symbol_style: parse_symbol_style(&options.symbol_style)?,
                date_style: parse_date_style(&options.date_style)?,
                colon_style: parse_colon_style(&options.colon_style)?,
                numeric_date_order: parse_numeric_date_order(&options.numeric_date_order)?,
                currency_symbol_policy: parse_currency_symbol_policy(
                    &options.currency_symbol_policy,
                )?,
                input_tolerance: parse_input_tolerance(&options.input_tolerance)?,
                asr_vocabulary: options.asr_vocabulary.clone(),
                vocabulary: options.vocabulary.clone(),
            })
        }
    }

    #[pymethods]
    impl PyNormalizeOptions {
        #[new]
        #[pyo3(signature = (preset = "default"))]
        fn new(preset: &str) -> PyResult<Self> {
            Ok(NormalizeOptions::preset(parse_preset(preset)?).into())
        }

        #[staticmethod]
        fn from_preset(preset: &str) -> PyResult<Self> {
            Self::new(preset)
        }

        fn copy(&self) -> Self {
            self.clone()
        }

        fn __repr__(&self) -> String {
            format!(
                "NormalizeOptions(range_style={:?}, date_style={:?}, input_tolerance={:?})",
                self.range_style, self.date_style, self.input_tolerance
            )
        }
    }

    /// A sentence or token with UTF-8 byte offsets into the input.
    #[pyclass(name = "Span", module = "ukrainian_tn", frozen, get_all, skip_from_py_object)]
    #[derive(Clone)]
    struct PySpan {
        start: usize,
        stop: usize,
        text: String,
    }

    #[pymethods]
    impl PySpan {
        fn __repr__(&self) -> String {
            format!("Span(start={}, stop={}, text={:?})", self.start, self.stop, self.text)
        }
    }

    impl From<rozpodil::Substring<'_>> for PySpan {
        fn from(span: rozpodil::Substring<'_>) -> Self {
            Self { start: span.start, stop: span.stop, text: span.text.to_owned() }
        }
    }

    /// An input span whose spoken interpretation may be ambiguous.
    #[pyclass(
        name = "UncertainSpan",
        module = "ukrainian_tn",
        frozen,
        get_all,
        skip_from_py_object
    )]
    #[derive(Clone)]
    struct PyUncertainSpan {
        start: usize,
        stop: usize,
        text: String,
        reason: String,
        category: String,
        severity: String,
    }

    #[pymethods]
    impl PyUncertainSpan {
        fn __repr__(&self) -> String {
            format!(
                "UncertainSpan(start={}, stop={}, text={:?}, category={:?}, severity={:?})",
                self.start, self.stop, self.text, self.category, self.severity
            )
        }
    }

    const fn category_name(category: UncertaintyCategory) -> &'static str {
        match category {
            UncertaintyCategory::AmbiguousAbbreviation => "ambiguous_abbreviation",
            UncertaintyCategory::BareNumber => "bare_number",
            UncertaintyCategory::Currency => "currency",
            UncertaintyCategory::Date => "date",
            UncertaintyCategory::Identifier => "identifier",
            UncertaintyCategory::ForeignWord => "foreign_word",
            UncertaintyCategory::MixedScript => "mixed_script",
            UncertaintyCategory::RomanNumeral => "roman_numeral",
            UncertaintyCategory::Unit => "unit",
            UncertaintyCategory::Web => "web",
            UncertaintyCategory::InvalidDate => "invalid_date",
            UncertaintyCategory::AmbiguousNumberGrouping => "ambiguous_number_grouping",
            UncertaintyCategory::Agreement => "agreement",
            UncertaintyCategory::Time => "time",
            UncertaintyCategory::Fraction => "fraction",
            UncertaintyCategory::Network => "network",
            UncertaintyCategory::Scientific => "scientific",
            UncertaintyCategory::Coordinate => "coordinate",
            UncertaintyCategory::ApproximateMatch => "approximate_match",
        }
    }

    const fn severity_name(severity: UncertaintySeverity) -> &'static str {
        match severity {
            UncertaintySeverity::Info => "info",
            UncertaintySeverity::Warning => "warning",
            UncertaintySeverity::Error => "error",
        }
    }

    impl From<uktextnorm::UncertainSpan> for PyUncertainSpan {
        fn from(span: uktextnorm::UncertainSpan) -> Self {
            Self {
                start: span.start,
                stop: span.stop,
                text: span.text,
                reason: span.reason,
                category: category_name(span.category).to_owned(),
                severity: severity_name(span.severity).to_owned(),
            }
        }
    }

    fn vocabulary_error(error: VocabularyError) -> PyErr {
        match error {
            VocabularyError::Io(error) => PyOSError::new_err(error.to_string()),
            error @ VocabularyError::Invalid { .. } => PyValueError::new_err(error.to_string()),
        }
    }

    /// Normalize Ukrainian text with the default options.
    #[pyfunction]
    #[pyo3(name = "normalize")]
    fn normalize_py(py: Python<'_>, text: String) -> String {
        py.detach(move || uktextnorm::normalize(&text))
    }

    /// Normalize text using a named preset.
    #[pyfunction]
    #[pyo3(name = "normalize_preset")]
    fn normalize_preset_py(py: Python<'_>, text: String, preset: &str) -> PyResult<String> {
        let preset = parse_preset(preset)?;
        Ok(py.detach(move || uktextnorm::normalize_preset(&text, preset)))
    }

    /// Normalize text using a mutable `NormalizeOptions` object.
    #[pyfunction]
    #[pyo3(name = "normalize_with")]
    fn normalize_with_py(
        py: Python<'_>,
        text: String,
        options: &Bound<'_, PyNormalizeOptions>,
    ) -> PyResult<String> {
        let options = NormalizeOptions::try_from(&*options.borrow())?;
        Ok(py.detach(move || uktextnorm::normalize_with(&text, &options)))
    }

    /// Split text into sentences and return their text and byte offsets.
    #[pyfunction]
    #[pyo3(name = "split_sentences")]
    fn split_sentences_py(py: Python<'_>, text: String) -> Vec<PySpan> {
        py.detach(move || rozpodil::split_sentences(&text).into_iter().map(PySpan::from).collect())
    }

    /// Split text into tokens and return their text and byte offsets.
    #[pyfunction]
    #[pyo3(name = "tokenize")]
    fn tokenize_py(py: Python<'_>, text: String) -> Vec<PySpan> {
        py.detach(move || rozpodil::tokenize(&text).into_iter().map(PySpan::from).collect())
    }

    /// Report ambiguous spans using the default options.
    #[pyfunction]
    #[pyo3(name = "flag_uncertain")]
    fn flag_uncertain_py(py: Python<'_>, text: String) -> Vec<PyUncertainSpan> {
        py.detach(move || {
            uktextnorm::flag_uncertain(&text).into_iter().map(PyUncertainSpan::from).collect()
        })
    }

    /// Report ambiguous spans after applying explicit option choices.
    #[pyfunction]
    #[pyo3(name = "flag_uncertain_with")]
    fn flag_uncertain_with_py(
        py: Python<'_>,
        text: String,
        options: &Bound<'_, PyNormalizeOptions>,
    ) -> PyResult<Vec<PyUncertainSpan>> {
        let options = NormalizeOptions::try_from(&*options.borrow())?;
        Ok(py.detach(move || {
            uktextnorm::flag_uncertain_with(&text, &options)
                .into_iter()
                .map(PyUncertainSpan::from)
                .collect()
        }))
    }

    /// Spell an integer in Ukrainian words.
    #[pyfunction]
    #[pyo3(name = "number_to_words")]
    fn number_to_words_py(py: Python<'_>, number: u64) -> String {
        py.detach(move || uktextnorm::number_to_words(number))
    }

    /// Read every ASCII digit separately.
    #[pyfunction]
    #[pyo3(name = "number_to_words_digit_by_digit")]
    fn number_to_words_digit_by_digit_py(py: Python<'_>, digits: String) -> String {
        py.detach(move || uktextnorm::number_to_words_digit_by_digit(&digits))
    }

    /// Spell an integer as an ordinal in the requested form.
    #[pyfunction]
    #[pyo3(name = "number_to_ordinal_words", signature = (number, form = "nom_m"))]
    fn number_to_ordinal_words_py(py: Python<'_>, number: u64, form: &str) -> PyResult<String> {
        let form = parse_ordinal_form(form)?;
        Ok(py.detach(move || uktextnorm::number_to_ordinal_words(number, form)))
    }

    /// Spell an integer in a grammatical case.
    #[pyfunction]
    #[pyo3(name = "number_to_words_case")]
    fn number_to_words_case_py(py: Python<'_>, number: u64, case: &str) -> PyResult<String> {
        let case = parse_grammatical_case(case)?;
        Ok(py.detach(move || uktextnorm::number_to_words_case(number, case)))
    }

    /// Normalize common abbreviations.
    #[pyfunction]
    #[pyo3(name = "normalize_abbreviations")]
    fn normalize_abbreviations_py(py: Python<'_>, text: String) -> String {
        py.detach(move || uktextnorm::normalize_abbreviations(&text))
    }

    /// Expand uppercase Ukrainian acronyms letter by letter.
    #[pyfunction]
    #[pyo3(name = "expand_abbreviations")]
    fn expand_abbreviations_py(py: Python<'_>, text: String) -> String {
        py.detach(move || uktextnorm::expand_abbreviations(&text))
    }

    /// Transliterate Latin text to Cyrillic.
    #[pyfunction]
    #[pyo3(name = "transliterate_to_cyrillic")]
    fn transliterate_to_cyrillic_py(py: Python<'_>, text: String) -> String {
        py.detach(move || uktextnorm::transliterate_to_cyrillic(&text))
    }

    /// Parse a two-column normalization vocabulary from TSV text.
    #[pyfunction]
    #[pyo3(name = "parse_vocabulary")]
    fn parse_vocabulary_py(source: &str) -> PyResult<HashMap<String, String>> {
        uktextnorm::parse_vocabulary(source).map_err(vocabulary_error)
    }

    /// Load a two-column normalization vocabulary from a TSV file.
    #[pyfunction]
    #[pyo3(name = "load_vocabulary_tsv")]
    fn load_vocabulary_tsv_py(path: &str) -> PyResult<HashMap<String, String>> {
        uktextnorm::load_vocabulary_tsv(path).map_err(vocabulary_error)
    }

    /// Parse a one-column ASR vocabulary from TSV text.
    #[pyfunction]
    #[pyo3(name = "parse_asr_vocabulary")]
    fn parse_asr_vocabulary_py(source: &str) -> PyResult<Vec<String>> {
        uktextnorm::parse_asr_vocabulary(source).map_err(vocabulary_error)
    }

    /// Load a one-column ASR vocabulary from a TSV file.
    #[pyfunction]
    #[pyo3(name = "load_asr_vocabulary_tsv")]
    fn load_asr_vocabulary_tsv_py(path: &str) -> PyResult<Vec<String>> {
        uktextnorm::load_asr_vocabulary_tsv(path).map_err(vocabulary_error)
    }

    #[pymodule_export]
    const MAX_SPELLED_NUMBER: u64 = uktextnorm::MAX_SPELLED_NUMBER;

    #[pymodule_export]
    #[allow(non_upper_case_globals)]
    const __version__: &str = env!("CARGO_PKG_VERSION");

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn options_round_trip() {
            let mut rust = NormalizeOptions::preset(NormalizePreset::TtsFriendly);
            rust.input_tolerance = InputTolerance::Asr;
            rust.vocabulary.insert("wifi".to_owned(), "вайфай".to_owned());

            let python = PyNormalizeOptions::from(rust.clone());
            let round_trip = NormalizeOptions::try_from(&python).unwrap();
            assert_eq!(round_trip, rust);
        }

        #[test]
        fn invalid_option_value_is_rejected() {
            let mut options = PyNormalizeOptions::from(NormalizeOptions::default());
            options.range_style = "invalid".to_owned();
            assert!(NormalizeOptions::try_from(&options).is_err());
        }

        #[test]
        fn spans_become_owned_python_values() {
            let text = "Раз. Два.";
            let spans: Vec<_> =
                rozpodil::split_sentences(text).into_iter().map(PySpan::from).collect();
            assert_eq!(spans.len(), 2);
            assert_eq!(spans[0].text, "Раз.");
        }
    }
}
