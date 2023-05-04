use brainfuck_interpeter::{BFResult, Program, BFError};



fn main_wrapper() -> BFResult<()> {
    
    let arguments: Vec<String> = std::env::args().collect();
    if arguments.len() < 2 {
        return Err(BFError::ArgError);
    }

    let mut program: Program = Program::build_from_file(arguments[1].to_owned())?;

    // Run based on mode of operation
    let arguments: Vec<String> = std::env::args().collect();
    if arguments.len() > 2 {
        if arguments[2] == "visualised" {
            program.diagnostic_run()?;
        } else {
            program.run()?;
        }
    } else {
        program.run()?;
    }

    Ok(())
}

fn main() {
    if let Err(smthng) = main_wrapper() {
        eprintln!("Generický error handler:\n{}", smthng);
    }
}
