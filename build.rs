use std::{env, fs, io, path::PathBuf};

fn main() {
    fs::create_dir_all(search().join("lib")).unwrap();
    add_cflags().unwrap();
    add_ldflags().unwrap();
    zlib::build().unwrap();
    caca::build().unwrap();
    caca::bindgen().unwrap();
}

fn add_cflags() -> io::Result<()> {
    let mut cflags = match env::var("CFLAGS") {
        Ok(cflags) => cflags.trim().to_owned(),
        Err(_) => "".to_owned(),
    };

    if !cflags.is_empty() {
        cflags.push(' ');
    }

    cflags.push_str(&format!("-I{}", search().join("include").display()));

    env::set_var("CFLAGS", cflags);

    Ok(())
}

fn add_ldflags() -> io::Result<()> {
    let mut ldflags = match env::var("LDFLAGS") {
        Ok(ldflags) => ldflags.trim().to_owned(),
        Err(_) => "".to_owned(),
    };

    if !ldflags.is_empty() {
        ldflags.push(' ');
    }

    ldflags.push_str(&format!("-L{}", search().join("lib").display()));

    env::set_var("LDFLAGS", ldflags);

    Ok(())
}

fn output() -> PathBuf {
    PathBuf::from(env::var("OUT_DIR").unwrap())
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

mod zlib {
    use std::{fs, io, process::Command};

    use cmake::Config;

    use super::{output, search};

    const VERSION: &'static str = "2.0.3";

    pub fn build() -> io::Result<()> {
        fetch()?;
        run_cmake()?;

        Ok(())
    }

    fn fetch() -> io::Result<()> {
        let output_base_path = output();
        let clone_dest_dir = format!("zlib-ng-{}", VERSION);
        let _ = std::fs::remove_dir_all(output_base_path.join(&clone_dest_dir));
        let status = Command::new("git")
            .current_dir(&output_base_path)
            .arg("clone")
            .arg("--depth=1")
            .arg("-b")
            .arg(VERSION)
            .arg("https://github.com/zlib-ng/zlib-ng")
            .arg(&clone_dest_dir)
            .status()?;

        if status.success() {
            Ok(())
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "fetch failed"))
        }
    }

    fn run_cmake() -> io::Result<()> {
        let path = fs::canonicalize(output().join(format!("zlib-ng-{}", VERSION))).unwrap();
        let _ = Config::new(path)
            .define("ZLIB_COMPAT", "ON")
            .define("ZLIB_ENABLE_TESTS", "OFF")
            .define("WITH_GZFILEOP", "ON")
            .define("WITH_OPTIM", "ON")
            .define("WITH_NEW_STRATEGIES", "ON")
            .define("WITH_NATIVE_INSTRUCTIONS", "OFF")
            .define("WITH_SANITIZER", "OFF")
            .define("WITH_FUZZERS", "OFF")
            .define("WITH_MAINTAINER_WARNINGS", "OFF")
            .define("WITH_CODE_COVERAGE", "OFF")
            .define("CMAKE_BUILD_TYPE", "Release")
            .out_dir(search())
            .build();

        fs::remove_dir_all(search().join("build"))?;

        println!("cargo:rustc-link-lib=static=z");

        Ok(())
    }
}

mod caca {
    use super::{includes, output, search};

    use std::{io, path::PathBuf, process::Command};

    pub fn build() -> io::Result<()> {
        fetch()?;
        bootstrap()?;
        configure()?;
        make()?;
        install()?;
        link();

        Ok(())
    }

    fn link() {
        println!(
            "cargo:rustc-link-search=native={}",
            search().join("lib").to_string_lossy()
        );

        println!("cargo:rustc-link-lib=static=caca");

        if cfg!(feature = "x11") {
            println!("cargo:rustc-link-lib=dylib=x");
        }
    }

    fn source() -> PathBuf {
        output().join("libcaca")
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

        macro_rules! enable {
            ($feat:expr) => {
                configure.arg(concat!("--enable-", $feat))
            };
        }

        macro_rules! disable {
            ($feat:expr) => {
                configure.arg(concat!("--disable-", $feat))
            };
        }

        macro_rules! feature {
            ($feat:expr) => {
                feature!($feat, $feat)
            };
            ($feat:expr, $name:expr) => {
                if cfg!(feature = $feat) {
                    enable!($name)
                } else {
                    disable!($name)
                }
            };
        }

        enable!("static");
        disable!("shared");

        disable!("kernel");
        disable!("vga");

        disable!("csharp");
        disable!("java");
        disable!("cxx");
        disable!("python");
        disable!("ruby");
        disable!("php");
        disable!("perl");
        disable!("debug");
        disable!("profiling");
        disable!("plugins");
        disable!("doc");
        disable!("cppunit");
        disable!("zzuf");
        disable!("dependency-tracking");
        disable!("silent-rules");
        disable!("slang");
        disable!("gl");
        disable!("network");
        disable!("imlib2");
        disable!("ncurses");

        #[cfg(windows)]
        feature!("conio");
        #[cfg(windows)]
        feature!("win32");
        #[cfg(all(unix, not(target_os = "macos")))]
        feature!("x11");
        if cfg!(feature = "x11") {
            configure.arg("--with-X");
        }
        #[cfg(target_os = "macos")]
        feature!("cocoa");

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

    pub fn bindgen() -> io::Result<()> {
        let mut builder = bindgen::Builder::default().ctypes_prefix("libc");

        builder = builder.header(includes().join("caca0.h").to_string_lossy());

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
}
