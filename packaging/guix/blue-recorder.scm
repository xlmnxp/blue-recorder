(define-module
  (blue-recorder)
  #:use-module
  ((guix licenses) #:prefix license:)
  #:use-module
  (guix build-system cargo)
  #:use-module
  (guix download)
  #:use-module
  (guix git-download)
  #:use-module
  (guix packages)
  #:use-module
  (gnu packages pkg-config)
  #:use-module
  (gnu packages bash)
  #:use-module
  (gnu packages glib)
  #:use-module
  (gnu packages gtk)
  #:use-module
  (gnu packages gstreamer)
  #:use-module
  (gnu packages gettext)
  #:use-module
  (gnu packages pulseaudio)
  #:use-module
  (gnu packages freedesktop)
  #:use-module
  (gnu packages video)
  #:use-module
  (gnu packages base)
  #:use-module
  (gnu packages autotools)
  #:use-module
  (gnu packages llvm)
  #:use-module
  (gnu packages xorg)
  #:use-module
  (gnu packages compression)
  #:use-module
  (guix gexp)
  #:use-module
  (gnu packages crates-windows)
  #:use-module
  (gnu packages crates-crypto)
  #:use-module
  (gnu packages crates-gtk)
  #:use-module
  (gnu packages crates-apple)
  #:use-module
  (gnu packages crates-graphics)
  #:use-module
  (gnu packages crates-io))

(define rust-atk-sys-0.15
  (package
    (name "rust-atk-sys")
    (version "0.15.1")
    (source
      (origin
        (method url-fetch)
        (uri (crate-uri "atk-sys" version))
        (file-name
          (string-append name "-" version ".tar.gz"))
        (sha256
          (base32
            "1dmg7aq3f533cczkhcyyp929daqpwdz1r5w9h3dhd3k9zf4v1bjq"))))
    (build-system cargo-build-system)
    (arguments
      `(#:cargo-inputs
        (("rust-glib-sys" ,rust-glib-sys-0.15)
         ("rust-gobject-sys" ,rust-gobject-sys-0.15)
         ("rust-libc" ,rust-libc-0.2)
         ("rust-system-deps" ,rust-system-deps-6))))
    (home-page "https://gtk-rs.org/")
    (synopsis "FFI bindings to libatk-1")
    (description
      "This package provides FFI bindings to libatk-1.")
    (license license:expat)))

(define rust-gdk-sys-0.15
  (package
    (name "rust-gdk-sys")
    (version "0.15.1")
    (source
      (origin
        (method url-fetch)
        (uri (crate-uri "gdk-sys" version))
        (file-name
          (string-append name "-" version ".tar.gz"))
        (sha256
          (base32
	    "121s0wk24kflj7m13g578gvqj5lcgdvimrdpgwbz81lg3s6a1rrj"))))
    (build-system cargo-build-system)
    (arguments
      `(#:cargo-inputs
        (("rust-cairo-sys-rs" ,rust-cairo-sys-rs-0.20)
         ("rust-gdk-pixbuf-sys" ,rust-gdk-pixbuf-sys-0.15)
         ("rust-gio-sys" ,rust-gio-sys-0.15)
         ("rust-glib-sys" ,rust-glib-sys-0.15)
         ("rust-gobject-sys" ,rust-gobject-sys-0.15)
         ("rust-libc" ,rust-libc-0.2)
         ("rust-pango-sys" ,rust-pango-sys-0.15)
         ("rust-pkg-config" ,rust-pkg-config-0.3)
         ("rust-system-deps" ,rust-system-deps-6))))
    (home-page "https://gtk-rs.org/")
    (synopsis "FFI bindings to libgdk-3")
    (description
      "This package provides FFI bindings to libgdk-3.")
    (license license:expat)))

(define rust-gdk4-0.4
  (package
    (name "rust-gdk4")
    (version "0.4.8")
    (source
      (origin
        (method url-fetch)
        (uri (crate-uri "gdk4" version))
        (file-name
          (string-append name "-" version ".tar.gz"))
        (sha256
          (base32
            "0xh8b3ms20xmmp2gkvrfmsljggy0s2avp2nnln2v09iwhk7vgasg"))))
    (build-system cargo-build-system)
    (arguments
      `(#:cargo-inputs
        (("rust-bitflags" ,rust-bitflags-1)
         ("rust-cairo-rs" ,rust-cairo-rs-0.15)
         ("rust-gdk-pixbuf" ,rust-gdk-pixbuf-0.15)
         ("rust-gdk4-sys" ,rust-gdk4-sys-0.4)
         ("rust-gio" ,rust-gio-0.15)
         ("rust-glib" ,rust-glib-0.15)
         ("rust-libc" ,rust-libc-0.2)
         ("rust-pango" ,rust-pango-0.15))))
    (home-page "https://gtk-rs.org/gtk4-rs")
    (synopsis "Rust bindings of the GDK 4 library")
    (description
      "This package provides Rust bindings of the GDK 4 library.")
    (license license:expat)))


(define rust-gdk4-sys-0.4
  (package
    (name "rust-gdk4-sys")
    (version "0.4.8")
    (source
      (origin
        (method url-fetch)
        (uri (crate-uri "gdk4-sys" version))
        (file-name
          (string-append name "-" version ".tar.gz"))
        (sha256
          (base32
	    "1wnfv62n9dmpzg9rpy3hj1aldpkkavyans9zzymsw02w9ysdrrzg"))))
    (build-system cargo-build-system)
    (arguments
      `(#:cargo-inputs
        (("rust-cairo-sys-rs" ,rust-cairo-sys-rs-0.20)
         ("rust-gdk-pixbuf-sys" ,rust-gdk-pixbuf-sys-0.15)
         ("rust-gio-sys" ,rust-gio-sys-0.15)
         ("rust-glib-sys" ,rust-glib-sys-0.15)
         ("rust-gobject-sys" ,rust-gobject-sys-0.15)
         ("rust-libc" ,rust-libc-0.2)
         ("rust-pango-sys" ,rust-pango-sys-0.15)
         ("rust-pkg-config" ,rust-pkg-config-0.3)
         ("rust-system-deps" ,rust-system-deps-6))))
    (home-page "https://gtk-rs.org/gtk4-rs")
    (synopsis "FFI bindings of GDK 4")
    (description
      "This package provides FFI bindings of GDK 4.")
    (license license:expat)))

(define rust-graphene-rs-0.15
  (package
    (name "rust-graphene-rs")
    (version "0.15.1")
    (source
      (origin
        (method url-fetch)
        (uri (crate-uri "graphene-rs" version))
        (file-name
          (string-append name "-" version ".tar.gz"))
        (sha256
          (base32
	    "0w2mz098dr8mlz18ssmlnln1x6c3byizqbc9kz4n5nzgpvxzjm3w"))))
    (build-system cargo-build-system)
    (arguments
      `(#:cargo-inputs
        (("rust-glib" ,rust-glib-0.15)
         ("rust-graphene-sys" ,rust-graphene-sys-0.15)
         ("rust-libc" ,rust-libc-0.2))))
    (home-page "https://gtk-rs.org/")
    (synopsis
      "Rust bindings for the Graphene library")
    (description
      "This package provides Rust bindings for the Graphene library.")
    (license license:expat)))

(define rust-graphene-sys-0.15
  (package
    (name "rust-graphene-sys")
    (version "0.15.10")
    (source
      (origin
        (method url-fetch)
        (uri (crate-uri "graphene-sys" version))
        (file-name
          (string-append name "-" version ".tar.gz"))
        (sha256
          (base32
	    "12h2qcdhvzxhkc75fqkky6rz212wp2yc6mgvk9cxz8bv6g3iysgs"))))
    (build-system cargo-build-system)
    (arguments
      `(#:cargo-inputs
        (("rust-glib-sys" ,rust-glib-sys-0.15)
         ("rust-libc" ,rust-libc-0.2)
         ("rust-pkg-config" ,rust-pkg-config-0.3)
         ("rust-system-deps" ,rust-system-deps-6))))
    (home-page "https://gtk-rs.org/")
    (synopsis "FFI bindings to libgraphene-1.0")
    (description
      "This package provides FFI bindings to libgraphene-1.0.")
    (license license:expat)))

(define rust-gsk4-0.4
  (package
    (name "rust-gsk4")
    (version "0.4.8")
    (source
      (origin
        (method url-fetch)
        (uri (crate-uri "gsk4" version))
        (file-name
          (string-append name "-" version ".tar.gz"))
        (sha256
          (base32
	    "1r0vnrgdpkavxkq67bgixcp72l4vz9dlk5nl72mb701j6c6h5s85"))))
    (build-system cargo-build-system)
    (arguments
      `(#:cargo-inputs
        (("rust-bitflags" ,rust-bitflags-1)
         ("rust-cairo-rs" ,rust-cairo-rs-0.15)
         ("rust-gdk4" ,rust-gdk4-0.4)
         ("rust-glib" ,rust-glib-0.15)
         ("rust-graphene-rs" ,rust-graphene-rs-0.15)
         ("rust-gsk4-sys" ,rust-gsk4-sys-0.4)
         ("rust-libc" ,rust-libc-0.2)
         ("rust-pango" ,rust-pango-0.15))))
    (home-page "https://gtk-rs.org/gtk4-rs")
    (synopsis "Rust bindings of the GSK 4 library")
    (description
      "This package provides Rust bindings of the GSK 4 library.")
    (license license:expat)))

(define rust-gsk4-sys-0.4
  (package
    (name "rust-gsk4-sys")
    (version "0.4.8")
    (source
      (origin
        (method url-fetch)
        (uri (crate-uri "gsk4-sys" version))
        (file-name
          (string-append name "-" version ".tar.gz"))
        (sha256
          (base32
	    "1sizv9dy5ch1nxmfmdb3xm35q10zr7fa4hw6hf650y00yv63kpbs"))))
    (build-system cargo-build-system)
    (arguments
      `(#:cargo-inputs
        (("rust-cairo-sys-rs" ,rust-cairo-sys-rs-0.20)
         ("rust-gdk4-sys" ,rust-gdk4-sys-0.4)
         ("rust-glib-sys" ,rust-glib-sys-0.15)
         ("rust-gobject-sys" ,rust-gobject-sys-0.15)
         ("rust-graphene-sys" ,rust-graphene-sys-0.15)
         ("rust-libc" ,rust-libc-0.2)
         ("rust-pango-sys" ,rust-pango-sys-0.15)
         ("rust-system-deps" ,rust-system-deps-6))))
    (home-page "https://gtk-rs.org/gtk4-rs")
    (synopsis "FFI bindings of GSK 4")
    (description
      "This package provides FFI bindings of GSK 4.")
    (license license:expat)))

(define rust-gtk-sys-0.15
  (package
    (name "rust-gtk-sys")
    (version "0.15.3")
    (source
      (origin
        (method url-fetch)
        (uri (crate-uri "gtk-sys" version))
        (file-name
          (string-append name "-" version ".tar.gz"))
        (sha256
          (base32
            "113wp3x7xh2zjv8i5pn3mcz77yr5zq8wm8260bv4g8nbhw2jzg6m"))))
    (build-system cargo-build-system)
    (arguments
      `(#:cargo-inputs
        (("rust-atk-sys" ,rust-atk-sys-0.15)
         ("rust-cairo-sys-rs" ,rust-cairo-sys-rs-0.20)
         ("rust-gdk-pixbuf-sys" ,rust-gdk-pixbuf-sys-0.15)
         ("rust-gdk-sys" ,rust-gdk-sys-0.15)
         ("rust-gio-sys" ,rust-gio-sys-0.15)
         ("rust-glib-sys" ,rust-glib-sys-0.15)
         ("rust-gobject-sys" ,rust-gobject-sys-0.15)
         ("rust-libc" ,rust-libc-0.2)
         ("rust-pango-sys" ,rust-pango-sys-0.15)
         ("rust-system-deps" ,rust-system-deps-6))))
    (home-page "https://gtk-rs.org/")
    (synopsis "FFI bindings to libgtk-3")
    (description
      "This package provides FFI bindings to libgtk-3.")
    (license license:expat)))

(define rust-gtk4-0.4
  (package
    (name "rust-gtk4")
    (version "0.4.9")
    (source
      (origin
        (method url-fetch)
        (uri (crate-uri "gtk4" version))
        (file-name
          (string-append name "-" version ".tar.gz"))
        (sha256
          (base32
            "1g5v9wwf9sgz9vx0vwfc3sxm9pm5cah3ypjy3daw6fvryapfb2jf"))))
    (build-system cargo-build-system)
    (arguments
      `(#:cargo-inputs
        (("rust-bitflags" ,rust-bitflags-1)
         ("rust-cairo-rs" ,rust-cairo-rs-0.15)
         ("rust-field-offset" ,rust-field-offset-0.3)
         ("rust-futures-channel"
          ,rust-futures-channel-0.3)
         ("rust-gdk-pixbuf" ,rust-gdk-pixbuf-0.15)
         ("rust-gdk4" ,rust-gdk4-0.4)
         ("rust-gio" ,rust-gio-0.15)
         ("rust-glib" ,rust-glib-0.15)
         ("rust-graphene-rs" ,rust-graphene-rs-0.15)
         ("rust-gsk4" ,rust-gsk4-0.4)
         ("rust-gtk4-macros" ,rust-gtk4-macros-0.4)
         ("rust-gtk4-sys" ,rust-gtk4-sys-0.4)
         ("rust-libc" ,rust-libc-0.2)
         ("rust-once-cell" ,rust-once-cell-1)
         ("rust-pango" ,rust-pango-0.15))))
    (home-page "https://gtk-rs.org/gtk4-rs")
    (synopsis "Rust bindings of the GTK 4 library")
    (description
      "This package provides Rust bindings of the GTK 4 library.")
    (license license:expat)))

(define rust-gtk4-macros-0.4
  (package
    (name "rust-gtk4-macros")
    (version "0.4.10")
    (source
      (origin
        (method url-fetch)
        (uri (crate-uri "gtk4-macros" version))
        (file-name
          (string-append name "-" version ".tar.gz"))
        (sha256
          (base32
            "0v42i8xpg9f84iq1d0k2sb7vh94n9v9rk7i7iq3579wi9ra0pfka"))))
    (build-system cargo-build-system)
    (arguments
      `(#:cargo-inputs
        (("rust-anyhow" ,rust-anyhow-1)
         ("rust-proc-macro-crate"
          ,rust-proc-macro-crate-1)
         ("rust-proc-macro-error"
          ,rust-proc-macro-error-1)
         ("rust-proc-macro2" ,rust-proc-macro2-1)
         ("rust-quick-xml" ,rust-quick-xml-0.22)
         ("rust-quote" ,rust-quote-1)
         ("rust-syn" ,rust-syn-1))))
    (home-page "https://gtk-rs.org/gtk4-rs")
    (synopsis "Macros helpers for GTK 4 bindings")
    (description
      "This package provides Macros helpers for GTK 4 bindings.")
    (license license:expat)))

(define rust-gtk4-sys-0.4
  (package
    (name "rust-gtk4-sys")
    (version "0.4.8")
    (source
      (origin
        (method url-fetch)
        (uri (crate-uri "gtk4-sys" version))
        (file-name
          (string-append name "-" version ".tar.gz"))
        (sha256
          (base32
            "0qqgxfbmygsl3xd3qal37cdz4ibfc0j9xxrzv9r7qjv3x9p01j2v"))))
    (build-system cargo-build-system)
    (arguments
      `(#:cargo-inputs
        (("rust-cairo-sys-rs" ,rust-cairo-sys-rs-0.20)
         ("rust-gdk-pixbuf-sys" ,rust-gdk-pixbuf-sys-0.15)
         ("rust-gdk4-sys" ,rust-gdk4-sys-0.4)
         ("rust-gio-sys" ,rust-gio-sys-0.15)
         ("rust-glib-sys" ,rust-glib-sys-0.15)
         ("rust-gobject-sys" ,rust-gobject-sys-0.15)
         ("rust-graphene-sys" ,rust-graphene-sys-0.15)
         ("rust-gsk4-sys" ,rust-gsk4-sys-0.4)
         ("rust-libc" ,rust-libc-0.2)
         ("rust-pango-sys" ,rust-pango-sys-0.15)
         ("rust-system-deps" ,rust-system-deps-6))))
    (home-page "https://gtk-rs.org/gtk4-rs")
    (synopsis "FFI bindings of GTK 4")
    (description
      "This package provides FFI bindings of GTK 4.")
    (license license:expat)))

(define-public blue-recorder
  (package
    (name "blue-recorder")
    (version "0.2.0")
    (source
      (origin
        (method git-fetch)
        (uri (git-reference
	      (url "https://github.com/xlmnxp/blue-recorder")
	      (commit "1cfa3bbb1b5ea845b3e4c51eba269745f0c3e271")))

	(snippet
         #~(begin (use-modules (guix build utils))
                  (substitute* "Cargo.toml"
                    (("gdk = \\{ git =.+")
                     "gdk = { version = \"0.7.3\", package = \"gdk4\" }\n")
		    (("rust-ini =.+")
		     "rust-ini = \"0.18.0\"\n")))) ; We have this version in guix already

        (file-name
         (git-file-name name version))
        (sha256
         (base32
	  "0fz5l1z5rq8gx2vhrpfnf5l5karlqa7m8fdwx7ixlvy5klywwa5y"))))
    (build-system cargo-build-system)
    (arguments
     `(#:cargo-build-flags '("--release")
       #:phases ,#~(modify-phases %standard-phases
				 (add-after 'install 'wrap-paths
					    (lambda _
					      (let* ((bin (string-append #$output "/bin"))
						     (name-version (string-append #$name "-" #$version))
						     (blue-recorder (string-append bin "/blue-recorder"))
						     (src (string-append #$output "/share/cargo/src/"))
						     (po (string-append src name-version "/po/"))
						     (data (string-append src name-version "/data/")))
						(wrap-program blue-recorder
							      `("PO_DIR" prefix (,po))
							      `("DATA_DIR" prefix (,data)))))))
       #:cargo-inputs
       (("rust-async-std" ,rust-async-std-1)
        ("rust-chrono" ,rust-chrono-0.4)
        ("rust-dark-light" ,rust-dark-light-1)
        ("rust-dirs" ,rust-dirs-4)
        ("rust-filename" ,rust-filename-0.1)
        ("rust-gdk-pixbuf" ,rust-gdk-pixbuf-0.9)
        ("rust-gdk4" ,rust-gdk4-0.7)
        ("rust-gettext-rs" ,rust-gettext-rs-0.7)
        ("rust-gio" ,rust-gio-0.15)
        ("rust-glib" ,rust-glib-0.10)
        ("rust-gstreamer" ,rust-gstreamer-0.20)
        ("rust-gtk-sys" ,rust-gtk-sys-0.15)
        ("rust-gtk4" ,rust-gtk4-0.4)
        ("rust-regex" ,rust-regex-1)
        ("rust-rust-ini" ,rust-rust-ini-0.18)
        ("rust-secfmt" ,rust-secfmt-0.1)
        ("rust-subprocess" ,rust-subprocess-0.2)
        ("rust-tempfile" ,rust-tempfile-3)
        ("rust-zbus" ,rust-zbus-3))))
    (native-inputs (list pkg-config
			 glib
			 graphene
			 gstreamer
			 gnu-gettext
			 libappindicator
			 xz))
    (inputs (list glib))
    (propagated-inputs (list ffmpeg
			     gtk
			     gtk+
			     xwininfo
			     libappindicator
			     pulseaudio))
    (home-page "https://github.com/xlmnxp/blue-recorder/")
    (synopsis "Simple Screen Recorder written in Rust based on Green Recorder")
    (description "A simple desktop recorder for Linux systems.
Built using GTK4 and ffmpeg.  It supports recording audio and video on almost all Linux
interfaces with support for Wayland display server on GNOME session.")
    (license license:gpl3)))

blue-recorder
