use crate::Progress;

impl<P, T, E> Progress<P, T, E> {
    /// Maps the error of the progress to a snafu error, with the previous error as the source.
    ///
    /// `context_fn` has to be a function that returns the context selector of that error.
    #[inline]
    pub fn snafu<C, F, E2>(self, context_fn: F) -> Progress<P, T, E2>
    where
        P: Clone,
        C: snafu::IntoError<E2, Source = E>,
        F: FnOnce(P) -> C,
        E2: std::error::Error + snafu::ErrorCompat,
    {
        self.map_err_with_pos(|e, pos| context_fn(pos).into_error(e))
    }

    /// Maps the error of the progress to a snafu leaf error.
    ///
    /// `context_fn` has to be a function that returns the context selector of that leaf error.
    #[inline]
    pub fn into_snafu_leaf<C, F, E2>(self, context_fn: F) -> Progress<P, T, E2>
    where
        P: Clone,
        C: snafu::IntoError<E2, Source = snafu::NoneError>,
        F: FnOnce(E, P) -> C,
        E2: std::error::Error + snafu::ErrorCompat,
    {
        self.map_err_with_pos(|e, pos| context_fn(e, pos).into_error(snafu::NoneError))
    }

    /// Replaces the error of the progress with a snafu leaf error.
    ///
    /// `context_fn` has to be a function that returns the context selector of that leaf error.
    ///
    /// If you get an error because `E` cannot be inferred, try using
    /// [`into_snafu_leaf`](Progress::into_snafu_leaf) instead to specify the type of the error
    /// (first `context_fn` parameter).
    #[inline]
    pub fn snafu_leaf<C, F, E2>(self, context_fn: F) -> Progress<P, T, E2>
    where
        P: Clone,
        C: snafu::IntoError<E2, Source = snafu::NoneError>,
        F: FnOnce(P) -> C,
        E2: std::error::Error + snafu::ErrorCompat,
    {
        self.map_err_with_pos(|_, pos| context_fn(pos).into_error(snafu::NoneError))
    }
}
