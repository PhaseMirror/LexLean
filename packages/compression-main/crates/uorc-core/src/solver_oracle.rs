/// The strict result of an SMT solver execution.
/// Follows Issue 1304 requirements where `unknown`/timeout are mapped to incomplete states.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum SmtResult {
    /// The formula is definitely satisfiable.
    Sat,
    /// The formula is definitely unsatisfiable.
    Unsat,
    /// The solver returned `unknown`, timed out, or crashed.
    Unknown(String),
}

/// A strictly typed SMT-LIB v2 request generator.
#[derive(Debug, Clone)]
pub struct SmtRequest {
    /// The logic fragment to set for the solver (e.g. `QF_BV`).
    pub logic: String,
    /// A list of assertions in SMT-LIB v2 syntax.
    pub assertions: Vec<String>,
}

impl SmtRequest {
    /// Create a new SMT request targeting a specific logic fragment (e.g. `QF_BV`).
    pub fn new(logic: &str) -> Self {
        Self {
            logic: logic.to_string(),
            assertions: Vec::new(),
        }
    }

    /// Add an assertion (in SMT-LIB syntax) to the request.
    pub fn assert(&mut self, assertion: &str) {
        self.assertions.push(assertion.to_string());
    }

    /// Generates the raw SMT-LIB v2 text that will be sent to the solver.
    pub fn to_smt2(&self) -> String {
        let mut script = format!("(set-logic {})\n", self.logic);
        for assertion in &self.assertions {
            script.push_str(&format!("(assert {})\n", assertion));
        }
        script.push_str("(check-sat)\n");
        script
    }
}

/// A strict parser for SMT solver stdout.
/// Only exact `sat\n` or `unsat\n` is accepted; anything else maps to `Unknown`.
pub fn parse_smt_output(stdout: &str) -> SmtResult {
    let trimmed = stdout.trim();
    if trimmed == "sat" {
        SmtResult::Sat
    } else if trimmed == "unsat" {
        SmtResult::Unsat
    } else {
        SmtResult::Unknown(trimmed.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smt_request_generation() {
        let mut req = SmtRequest::new("QF_BV");
        req.assert("(= x #x00000001)");
        req.assert("(bvugt x #x00000000)");

        let smt2 = req.to_smt2();
        assert!(smt2.contains("(set-logic QF_BV)"));
        assert!(smt2.contains("(assert (= x #x00000001))"));
        assert!(smt2.contains("(check-sat)"));
    }

    #[test]
    fn test_strict_smt_parsing() {
        // Issue 1304: Handle unknown/timeout/unavailable as incomplete, not success.
        
        assert_eq!(parse_smt_output("sat\n"), SmtResult::Sat);
        assert_eq!(parse_smt_output("unsat\r\n"), SmtResult::Unsat);
        
        // Valid keyword but followed by error output => Unknown
        assert_eq!(
            parse_smt_output("unknown\n(error \"timeout\")"), 
            SmtResult::Unknown("unknown\n(error \"timeout\")".into())
        );

        // Blank or garbage output => Unknown
        assert_eq!(
            parse_smt_output(""), 
            SmtResult::Unknown("".into())
        );
        assert_eq!(
            parse_smt_output("segmentation fault"), 
            SmtResult::Unknown("segmentation fault".into())
        );
    }
}
