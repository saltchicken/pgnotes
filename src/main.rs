mod app;

use app::App;
use std::io;

fn main() -> io::Result<()> {
    let mut app = App::new()?;
    app.run()?;
    Ok(())
}
