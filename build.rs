use std::{env, io, path::PathBuf, process::Command};

fn main() {
    build().unwrap();
    link();
    bindgen().unwrap();
}

fn build() -> io::Result<()> {
    fetch()?;
    bootstrap()?;
    configure()?;
    make()?;
    install()?;

    Ok(())
}

fn link() {
    println!(
        "cargo:rustc-link-search=native={}",
        search().join("lib").to_string_lossy()
    );

    println!("cargo:rustc-link-lib=dylib=caca");
}

fn output() -> PathBuf {
    PathBuf::from(env::var("OUT_DIR").unwrap())
}

fn source() -> PathBuf {
    output().join("libcaca")
}

fn search() -> PathBuf {
    let mut absolute = env::current_dir().unwrap();
    absolute.push(&output());
    absolute.push("dist");

    absolute
}

fn includes() -> PathBuf {
    search().join("include")
}

fn fetch() -> io::Result<()> {
    let output_base_path = output();
    let clone_dest_dir = "libcaca".to_owned();
    let _ = std::fs::remove_dir_all(output_base_path.join(&clone_dest_dir));
    let status = Command::new("git")
        .current_dir(&output_base_path)
        .arg("clone")
        .arg("--depth=1")
        .arg("https://github.com/cacalabs/libcaca")
        .arg(&clone_dest_dir)
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(io::Error::new(io::ErrorKind::Other, "fetch failed"))
    }
}

fn bootstrap() -> io::Result<()> {
    let status = Command::new("./bootstrap")
        .current_dir(&source())
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(io::Error::new(io::ErrorKind::Other, "fetch failed"))
    }
}

fn configure() -> io::Result<()> {
    let mut configure = Command::new("./configure");
    configure.current_dir(&source());
    configure.arg(format!("--prefix={}", search().to_string_lossy()));

    configure.arg("--enable-static");
    configure.arg("--disable-kernel");
    configure.arg("--disable-vga");
    configure.arg("--disable-csharp");
    configure.arg("--disable-java");
    configure.arg("--disable-cxx");
    configure.arg("--disable-python");
    configure.arg("--disable-ruby");
    configure.arg("--disable-php");
    configure.arg("--disable-perl");
    configure.arg("--disable-debug");
    configure.arg("--disable-profiling");
    configure.arg("--disable-plugins");
    configure.arg("--disable-doc");
    configure.arg("--disable-cppunit");
    configure.arg("--disable-zzuf");
    configure.arg("--disable-dependency-tracking");
    configure.arg("--disable-shared");
    configure.arg("--disable-silent-rules");

    macro_rules! flag {
        ($conf:expr, $feat:expr, $name:expr) => {
            if env::var(concat!("CARGO_FEATURE_", $feat)).is_ok() {
                $conf.arg(concat!("--enable-", $name));
            } else {
                $conf.arg(concat!("--disable-", $name));
            }
        };
    }

    flag!(configure, "NCURSES", "ncurses");
    flag!(configure, "SLANG", "slang");
    flag!(configure, "GL", "gl");
    flag!(configure, "NETWORK", "network");
    flag!(configure, "IMLIB2", "imlib2");
    #[cfg(windows)]
    flag!(configure, "CONIO", "conio");
    #[cfg(windows)]
    flag!(configure, "WIN32", "win32");
    #[cfg(all(unix, not(target_os = "macos")))]
    flag!(configure, "X11", "x11");
    #[cfg(target_os = "macos")]
    flag!(configure, "COCOA", "cocoa");

    let output = configure
        .output()
        .unwrap_or_else(|_| panic!("{:?} failed", configure));
    if !output.status.success() {
        println!("configure: {}", String::from_utf8_lossy(&output.stdout));

        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "configure failed {}",
                String::from_utf8_lossy(&output.stderr)
            ),
        ));
    }

    Ok(())
}

fn make() -> io::Result<()> {
    if !Command::new("make")
        .arg("-j")
        .arg(num_cpus::get().to_string())
        .current_dir(&source())
        .status()?
        .success()
    {
        return Err(io::Error::new(io::ErrorKind::Other, "make failed"));
    }

    Ok(())
}

fn install() -> io::Result<()> {
    if !Command::new("make")
        .current_dir(&source())
        .arg("install")
        .status()?
        .success()
    {
        return Err(io::Error::new(io::ErrorKind::Other, "make install failed"));
    }

    Ok(())
}

fn bindgen() -> io::Result<()> {
    let mut builder = bindgen::Builder::default().ctypes_prefix("libc");

    builder = builder
        //.header(includes().join("caca_types.h").to_string_lossy())
        .header(includes().join("caca0.h").to_string_lossy())
        //.header(includes().join("caca.h").to_string_lossy())
        ;

    #[cfg(windows)]
    if env::var("CARGO_FEATURE_CONIO").is_ok() {
        builder = builder.header(includes().join("caca_conio.h").to_string_lossy());
    }

    let bindings = builder.generate().expect("Unable to generate bindings");

    bindings
        .write_to_file(output().join("bindings.rs"))
        .expect("Couldn't write bindings.");

    Ok(())
}
