use std::str::FromStr;
use std::fmt;

type Atom = u32;

#[derive(Debug)]
struct Literal(i32);

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
struct Clause(Vec<Literal>);

impl FromStr for Clause {
    type Err = String;
    fn from_str(input: &str) -> Result<Self, String> {
        let mut lits: Vec<Literal> = Vec::new();
        for word in input.split(' ') {
            match word.parse::<i32>() {
                Err(e) => return Err(format!("Could not parse literal: {}", e.to_string())),
                Ok(0) => break,
                Ok(value) => lits.push(Literal(value)),
            };
        }
        Ok(Clause(lits))
    }
}

impl fmt::Display for Clause {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for lit in &self.0 {
            write!(f, "{} ", lit)?;
        }
        write!(f, "0")
    }
}

#[derive(Debug)]
struct Formula(Vec<Clause>);

impl FromStr for Formula {
    type Err = String;
    fn from_str(input: &str) -> Result<Self, String> {
        let mut clauses: Vec<Clause> = Vec::new();
        for line in input.lines() {
            match line.chars().next() {
                Some('p') => {} // Formula size. Can ignore
                Some('c') => {} // comment
                None => {}
                _ => clauses.push(line.parse::<Clause>()?),
            }
        }
        Ok(Self(clauses))
    }
}

impl fmt::Display for Formula {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Calculate number of variables (max absolute value of any literal)
        let num_vars = self.0.iter()
            .flat_map(|clause| clause.0.iter())
            .map(|lit| lit.0.abs())
            .max()
            .unwrap_or(0);

        let num_clauses = self.0.len();

        writeln!(f, "p cnf {} {}", num_vars, num_clauses)?;
        for clause in &self.0 {
            writeln!(f, "{}", clause)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
enum Reason {
    Decision { level: u32 },
    Implied { level: u32, antecedent: u32 },
}

#[derive(Debug)]
struct VarData {
    value: bool,
    reason: Reason,
}

#[derive(Debug)]
struct Valuation(Vec<Option<VarData>>);

#[derive(Debug)]
struct Lilsat {
    formula: Formula,
    valuation: Formula,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <cnf-file>", args[0]);
        std::process::exit(1);
    }

    let filename = &args[1];

    let contents = std::fs::read_to_string(filename)
        .unwrap_or_else(|err| {
            eprintln!("Error reading file '{}': {}", filename, err);
            std::process::exit(1);
        });

    let formula: Formula = contents.parse()
        .unwrap_or_else(|err| {
            eprintln!("Error parsing CNF: {}", err);
            std::process::exit(1);
        });

    println!("{}", formula);
}
