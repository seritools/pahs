/// Tracks the result of a parser: where it is and if it is successful.
///
/// On success, some value has been parsed. On failure, nothing has
/// been parsed and the error indicates the reason for the failure.
/// The returned point indicates where to next start parsing, often
/// unchanged on failure.
#[must_use]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Progress<P, T, E> {
    /// The current location.
    pub pos: P,
    /// If the point indicates the location of a successful or failed parse.
    pub status: Result<T, E>,
}

impl<P, T, E> Progress<P, T, E> {
    /// Creates a new `Progress` value indicating a successful parse.
    #[inline]
    pub fn success(pos: P, val: T) -> Progress<P, T, E> {
        Progress {
            pos,
            status: Ok(val),
        }
    }

    /// Creates a new `Progress` value indicating a failed parse.
    #[inline]
    pub fn failure(pos: P, err: E) -> Progress<P, T, E> {
        Progress {
            pos,
            status: Err(err),
        }
    }

    /// Maps the success value, if there is one.
    ///
    /// If the current position is needed while mapping,
    /// see [`map_with_pos`](Progress::map_with_pos) instead.
    #[inline]
    pub fn map<F, T2>(self, f: F) -> Progress<P, T2, E>
    where
        F: FnOnce(T) -> T2,
    {
        Progress {
            pos: self.pos,
            status: self.status.map(f),
        }
    }

    /// Maps the success value, if there is one.
    #[inline]
    pub fn map_with_pos<F, T2>(self, f: F) -> Progress<P, T2, E>
    where
        F: FnOnce(T, P) -> T2,
        P: Clone,
    {
        let pos = self.pos.clone();

        Progress {
            pos: self.pos,
            status: self.status.map(|val| f(val, pos)),
        }
    }

    /// Maps the success value, if there is one, potentially
    /// converting into a failure.
    ///
    /// If the current position is needed while mapping,
    /// see [`and_then_with_pos`](Progress::and_then_with_pos) instead.
    #[inline]
    pub fn and_then<T2, F>(self, restore_to: P, f: F) -> Progress<P, T2, E>
    where
        F: FnOnce(T) -> Result<T2, E>,
    {
        match self.status.and_then(f) {
            s @ Ok(..) => Progress {
                pos: self.pos,
                status: s,
            },
            s @ Err(..) => Progress {
                pos: restore_to,
                status: s,
            },
        }
    }

    /// Maps the success value, if there is one, potentially
    /// converting into a failure.
    #[inline]
    pub fn and_then_with_pos<T2, F>(self, restore_to: P, f: F) -> Progress<P, T2, E>
    where
        F: FnOnce(T, P) -> Result<T2, E>,
        P: Clone,
    {
        let pos = self.pos.clone();
        match self.status.and_then(|val| f(val, pos)) {
            s @ Ok(..) => Progress {
                pos: self.pos,
                status: s,
            },
            s @ Err(..) => Progress {
                pos: restore_to,
                status: s,
            },
        }
    }

    /// Maps the failure value, if there is one.
    ///
    /// If the current position is needed while mapping,
    /// see [`map_err_with_pos`](Progress::map_err_with_pos) instead.
    #[inline]
    pub fn map_err<E2, F>(self, f: F) -> Progress<P, T, E2>
    where
        F: FnOnce(E) -> E2,
    {
        Progress {
            pos: self.pos,
            status: self.status.map_err(f),
        }
    }

    /// Maps the failure value, if there is one.
    #[inline]
    pub fn map_err_with_pos<E2, F>(self, f: F) -> Progress<P, T, E2>
    where
        F: FnOnce(E, P) -> E2,
        P: Clone,
    {
        let pos = self.pos.clone();
        Progress {
            pos: self.pos,
            status: self.status.map_err(|e| f(e, pos)),
        }
    }

    /// Returns the value and the current position on success,
    /// or resets the position and returns `None` on failure.
    #[inline]
    pub fn into_optional(self, reset_to: P) -> (P, Option<T>) {
        match self {
            Progress {
                pos,
                status: Ok(val),
            } => (pos, Some(val)),
            Progress {
                status: Err(..), ..
            } => (reset_to, None),
        }
    }

    /// Unwraps itself into the position and the successfully parsed value.
    ///
    /// Panics if the parse status is an `Err`.
    #[inline]
    pub fn unwrap(self) -> (P, T) {
        if let Progress {
            status: Ok(val),
            pos,
        } = self
        {
            (pos, val)
        } else {
            panic!("called `unwrap` on error `Progress`")
        }
    }

    /// Unwraps itself into the point and the error.
    ///
    /// Panics if the parse status is not an `Err`.
    #[inline]
    pub fn unwrap_err(self) -> (P, E) {
        if let Progress {
            status: Err(e),
            pos,
        } = self
        {
            (pos, e)
        } else {
            panic!("called `unwrap_err` on non-error `Progress`")
        }
    }

    /// Converts this progress into a position and a result.
    #[inline]
    pub fn finish(self) -> (P, Result<T, E>) {
        (self.pos, self.status)
    }
}
