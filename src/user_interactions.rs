use std::io::{stdin, stdout, Write};



pub fn ask_user_action() -> Result<String, std::io::Error> {
    // Asking what the user wants to do
    println!(
        "What do you want to do?\n
            \t1 - Install a mod (you have to provide a zip folder of the mod)
            \t2 - Uninstall a mod (you have to type the name of the mod)
            \t3 - List all mods
            \t0 - Close ATA"
    );
    print!("\nInsert a number: ");
    stdout().flush()?;

    // Getting the user's action's id
    let mut answer = String::new();
    stdin().read_line(&mut answer)?;
    Ok(answer.trim().to_string())
}

