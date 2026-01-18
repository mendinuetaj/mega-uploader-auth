use figlet_rs::FIGfont;
use std::io::Write;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

pub fn print_banner() {
    let standard_font = FIGfont::standard().unwrap();
    let figure = standard_font.convert("Mega Auth");

    let mut stdout = StandardStream::stdout(ColorChoice::Auto);

    // Print FIGlet banner in Cyan
    let _ = stdout.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)).set_bold(true));
    if let Some(fig) = figure {
        writeln!(&mut stdout, "{}", fig).unwrap();
    }

    // Package Info
    const PKG_NAME: &str = env!("CARGO_PKG_NAME");
    const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
    const PKG_DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

    let _ = stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)));
    writeln!(&mut stdout, "Package: {}", PKG_NAME).unwrap();
    writeln!(&mut stdout, "Version: {}", PKG_VERSION).unwrap();
    writeln!(&mut stdout, "Description: {}", PKG_DESCRIPTION).unwrap();

    let _ = stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)));
    writeln!(&mut stdout, "\nAvailable Endpoints:").unwrap();

    let _ = stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)));
    writeln!(&mut stdout, "  - GET /                 (API Info)").unwrap();
    writeln!(&mut stdout, "  - POST /auth/cli/start  (CLI Auth Start)").unwrap();
    writeln!(&mut stdout, "  - GET /auth/cli/status  (CLI Auth Status)").unwrap();
    writeln!(&mut stdout, "  - POST /auth/cli/renew  (CLI Auth Renew)").unwrap();

    let _ = stdout.set_color(ColorSpec::new().set_fg(Some(Color::Magenta)));
    writeln!(&mut stdout, "\nTechnologies: Redis + BB8, AWS STS, Actix Web 4").unwrap();

    let _ = stdout.reset();
    writeln!(&mut stdout, "------------------------------------------------------------").unwrap();
}
