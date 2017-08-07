use super::*;
use odbc_sys::*;
use std::marker::PhantomData;

/// A `Statement` is most easily thought of as an SQL statement, such as `SELECT * FROM Employee`.
///
/// * The statement's state
/// * The current statement-level diagnostics
/// * The addresses of the application variables bound to the statement's parameters and result set
///   columns
/// * The current settings of each statement attribute
///
/// See [Statement Handles][1]
/// [1]: https://docs.microsoft.com/sql/odbc/reference/develop-app/statement-handles
#[derive(Debug)]
pub struct Statement<'con, C = NoResult> {
    cursor: PhantomData<C>,
    handle: HStmt<'con>,
}

/// Cursor state of `Statement`. A statement is likely to enter this state after executing e.g a
/// `SELECT` query.
#[derive(Debug)]
#[allow(missing_copy_implementations)]
pub enum HasResult {}
/// Cursor state of `Statement`. A statement is likely to enter this state after executing e.g. a
/// `CREATE TABLE` statement.
#[derive(Debug)]
#[allow(missing_copy_implementations)]
pub enum NoResult {}

impl<'con, C> Statement<'con, C> {
    /// Provides access to the raw ODBC Statement Handle
    pub unsafe fn as_raw(&self) -> SQLHSTMT {
        self.handle.as_raw()
    }

    fn transit<C2>(self) -> Statement<'con, C2> {
        Statement {
            handle: self.handle,
            cursor: PhantomData,
        }
    }
}

impl<'con> Statement<'con, NoResult> {
    /// Allocates a new `Statement`
    pub fn with_parent(parent: &'con Connection<Connected>) -> Return<Self> {
        HStmt::allocate(parent.as_hdbc()).map(|handle| {
            Statement {
                handle,
                cursor: PhantomData,
            }
        })
    }

    /// Executes a preparable statement, using the current values of the parametr marker variables.
    ///
    /// * See [SQLExecDirect][1]
    /// * See [Direct Execution][2]
    /// [1]: https://docs.microsoft.com/sql/odbc/reference/syntax/sqlexecdirect-function
    /// [2]: https://docs.microsoft.com/sql/odbc/reference/develop-app/direct-execution-odbc
    pub fn exec_direct<T>(
        mut self,
        statement_text: &T,
    ) -> ReturnNoData<Statement<'con, HasResult>, Statement<'con, NoResult>>
    where
        T: SqlStr + ?Sized,
    {
        match self.handle.exec_direct(statement_text) {
            ReturnNoData::Success(()) => ReturnNoData::Success(self.transit()),
            ReturnNoData::Info(()) => ReturnNoData::Info(self.transit()),
            ReturnNoData::NoData(()) => ReturnNoData::NoData(self.transit()),
            ReturnNoData::Error(()) => ReturnNoData::Error(self.transit()),
        }
    }
}

impl<'con, C> Diagnostics for Statement<'con, C> {
    fn diagnostics(&self, rec_number: SQLSMALLINT, message_text: &mut [SQLCHAR]) -> DiagReturn {
        self.handle.diagnostics(rec_number, message_text)
    }
}