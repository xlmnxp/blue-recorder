(define-module (blue-recorder)
               #:use-module (guix packages)
               #:use-module (guix download)
               #:use-module (guix git-download)
               #:use-module (guix build-system cargo)
               #:use-module (guix build-system gnu)
               #:use-module (gnu packages crates-graphics)
               #:use-module (gnu packages crates-io)
               #:use-module (gnu packages crates-gtk)
               #:use-module (guix build-system copy)
               #:use-module (gnu packages freedesktop)
               #:use-module ((guix licenses) #:prefix license:)
               #:use-module (gnu packages llvm)
               #:use-module (gnu packages game-development)
               #:use-module (gnu packages gettext)
               #:use-module (gnu packages glib)
               #:use-module (gnu packages gtk)
               #:use-module (gnu packages pkg-config)
               #:use-module (gnu packages pulseaudio)
               #:use-module (gnu packages video)
               #:use-module (gnu packages xorg))

(define-public rust-rust-ini-0.16
  (package
    (name "rust-rust-ini")
    (version "0.16.0")
    (source
     (origin
       (method url-fetch)
       (uri (crate-uri "rust-ini" version))
       (file-name (string-append name "-" version ".tar.gz"))
       (sha256
        (base32 "0qnyp96kslw5ivcf5a7dhiwbqpb72sh8rq40m81vbkiim54z5nyw"))))
    (build-system cargo-build-system)
    (arguments 
     `(#:skip-build? #t
       #:cargo-inputs
       (("rust-ordered-multimap" ,rust-ordered-multimap-0.3))))
    (home-page "https://github.com/zonyitoo/rust-ini")
    (synopsis "INI configuration file parsing library in Rust")
    (description
     "This package is an INI configuration file parsing library in Rust.")
    (license license:expat)))

(define-public rust-subprocess-0.2
  (package
    (name "rust-subprocess")
    (version "0.2.7")
    (source
     (origin
       (method url-fetch)
       (uri (crate-uri "subprocess" version))
       (file-name (string-append name "-" version ".tar.gz"))
       (sha256
        (base32 "1i6kbwggg36fwspybkyn9kwsy5azhg9g22aic2lrnlm2khgq0jrk"))))
    (build-system cargo-build-system)
    (arguments
     `(#:cargo-inputs
       (("rust-libc" ,rust-libc-0.2)
        ("rust-winapi" ,rust-winapi-0.3))
       #:cargo-development-inputs
       (("rust-tempdir" ,rust-tempdir-0.3)
        ("rust-lazy-static" ,rust-lazy-static-1))))
    (home-page "https://github.com/hniksic/rust-subprocess")
    (synopsis "Execution of child processes and pipelines.")
    (description
     "Execution of child processes and pipelines, inspired by Python's subprocess
module, with Rust-specific extensions.")
    (license (list license:asl2.0 license:expat))))

(define-public rust-enumflags2-derive-0.6
  (package
    (name "rust-enumflags2-derive")
    (version "0.6.4")
    (source
     (origin
       (method url-fetch)
       (uri (crate-uri "enumflags2-derive" version))
       (file-name (string-append name "-" version ".tar.gz"))
       (sha256
        (base32 "1kkcwi4n76bi1c16ms00dyk4d393gdf29kpr4k9zsn5z7m7fjvll"))))
    (build-system cargo-build-system)
    (arguments
     `(#:cargo-inputs
       (("rust-syn" ,rust-syn-1)
        ("rust-quote" ,rust-quote-1)
        ("rust-proc-macro2" ,rust-proc-macro2-1))))
    (home-page "https://github.com/NieDzejkob/enumflags2")
    (synopsis "Implements the classic bitflags datastructure.")
    (description
     "Enumflags2 implements the classic bitflags datastructure.")
    (license (list license:asl2.0 license:expat))))

(define-public rust-enumflags2-0.6
  (package
    (name "rust-enumflags2")
    (version "0.6.4")
    (source
     (origin
       (method url-fetch)
       (uri (crate-uri "enumflags2" version))
       (file-name (string-append name "-" version ".tar.gz"))
       (sha256
        (base32 "182xd6cxxmadx1axnz6x73d12pzgwkc712zq2lxd4z1k48lxij43"))))
    (build-system cargo-build-system)
    (arguments
     `(#:cargo-inputs
       (("rust-enumflags2-derive" ,rust-enumflags2-derive-0.6)
        ("rust-serde" ,rust-serde-1))))
    (home-page "https://github.com/NieDzejkob/enumflags2")
    (synopsis "Implements the classic bitflags datastructure.")
    (description
     "Enumflags2 implements the classic bitflags datastructure.")
    (license (list license:asl2.0 license:expat))))

(define-public rust-simple-logger-1
  (package
    (name "rust-simple-logger")
    (version "1.11.0")
    (source
     (origin
       (method url-fetch)
       (uri (crate-uri "simple-logger" version))
       (file-name (string-append name "-" version ".tar.gz"))
       (sha256
        (base32 "166iy6lxkf23am37aiyn104j8wfxmz59mp4r2i51vb9y15yg2myd"))))
    (build-system cargo-build-system)
    (arguments
     `(#:cargo-inputs
       (("rust-chrono" ,rust-chrono-0.4)
        ("rust-colored" ,rust-colored-1)
        ("rust-log" ,rust-log-0.4))))
    (home-page "https://github.com/borntyping/rust-simple-logger")
    (synopsis "A logger that prints all messages with a readable output format.")
    (description
     "A logger that prints all messages with a readable output format.")
    (license license:expat)))

(define-public rust-serde-xml-rs-0.4
  (package
    (name "rust-serde-xml-rs")
    (version "0.4.1")
    (source
     (origin
       (method url-fetch)
       (uri (crate-uri "serde-xml-rs" version))
       (file-name (string-append name "-" version ".tar.gz"))
       (sha256
        (base32 "1ykx1xkfd59gf0ijnp93xhpd457xy4zi8xv2hrr0ikvcd6h1pgzh"))))
    (build-system cargo-build-system)
    (arguments
     `(#:cargo-test-flags '("--release" "--" "--skip=doctype")
       #:cargo-inputs
       (("rust-log" ,rust-log-0.4)
        ("rust-serde" ,rust-serde-1)
        ("rust-xml-rs" ,rust-xml-rs-0.8)
        ("rust-thiserror" ,rust-thiserror-1))
       #:cargo-development-inputs
       (("rust-simple-logger" ,rust-simple-logger-1)
        ("rust-serde-derive" ,rust-serde-derive-1)
        ("rust-docmatic" ,rust-docmatic-0.1))))
    (home-page "https://github.com/RReverser/serde-xml-rs")
    (synopsis "Deserializer for Serde")
    (description
     "xml-rs based deserializer for Serde (compatible with 0.9+)")
    (license license:expat)))

(define-public rust-zvariant-derive-2
  (package
    (name "rust-zvariant-derive")
    (version "2.3.0")
    (source
     (origin
       (method url-fetch)
       (uri (crate-uri "zvariant-derive" version))
       (file-name (string-append name "-" version ".tar.gz"))
       (sha256
        (base32 "1kba1ynzcfch41wnvyn7y53rq6h8f7k6mwhwkdqra9m2yxk7qclg"))))
    (build-system cargo-build-system)
    (arguments
     `(#:cargo-inputs
       (("rust-proc-macro2" ,rust-proc-macro2-1)
        ("rust-syn" ,rust-syn-1)
        ("rust-quote" ,rust-quote-1)
        ("rust-proc-macro-crate" ,rust-proc-macro-crate-0.1))
       #:cargo-development-inputs
       (("rust-zvariant" ,rust-zvariant-2)
        ("rust-enumflags2" ,rust-enumflags2-0.6)
        ("rust-serde" ,rust-serde-1)
        ("rust-serde-repr" ,rust-serde-repr-0.1))))
    (home-page "https://gitlab.freedesktop.org/levans/zbus")
    (synopsis "A Rust API for D-Bus communication.")
    (description
     "A Rust API for D-Bus communication. The aim is to provide a safe and simple high- and low-level API akin to
GDBus, that doesn't depend on C libraries.")
    (license license:expat)))

(define-public rust-zvariant-2
  (package
    (name "rust-zvariant")
    (version "2.3.0")
    (source
     (origin
       (method url-fetch)
       (uri (crate-uri "zvariant" version))
       (file-name (string-append name "-" version ".tar.gz"))
       (sha256
        (base32 "19014ayd9k151zhxxm0sjqp5psjrjcmd2qb01narmmpgv5wbb2yc"))))
    (build-system cargo-build-system)
    (arguments
     `(#:tests? #f
       #:cargo-inputs
       (("rust-byteorder" ,rust-byteorder-1)
        ("rust-serde" ,rust-serde-1)
        ("rust-criterion" ,rust-criterion-0.3)
        ("rust-enumflags2" ,rust-enumflags2-0.6)
        ("rust-zvariant-derive" ,rust-zvariant-derive-2))
       #:cargo-development-inputs
       (("rust-serde-json" ,rust-serde-json-1)
        ("rust-serde-repr" ,rust-serde-repr-0.1)
        ("rust-glib" ,rust-glib-0.10)
        ("rust-rand" ,rust-rand-0.7))))
    (home-page "https://gitlab.freedesktop.org/levans/zbus")
    (synopsis "A Rust API for D-Bus communication.")
    (description
     "A Rust API for D-Bus communication. The aim is to provide a safe and simple high- and low-level API akin to
GDBus, that doesn't depend on C libraries.")
    (license license:expat)))

(define-public rust-ntest-0.7
  (package
    (name "rust-ntest")
    (version "0.7.1")
    (source
     (origin
       (method url-fetch)
       (uri (crate-uri "ntest" version))
       (file-name (string-append name "-" version ".tar.gz"))
       (sha256
        (base32
         "0kkk30ixlvfc97gpdjfv5584qmkhgx3mnphqf6rl2sxfckzxrg5k"))))
    (build-system cargo-build-system)
    (arguments
     `(#:cargo-inputs
       (("rust-ntest-test-cases" ,rust-ntest-test-cases-0.7)
        ("rust-ntest-timeout" ,rust-ntest-timeout-0.7)
        ("rust-ntest-proc-macro-helper" ,rust-ntest-proc-macro-helper-0.7)
        ("rust-timebomb" ,rust-timebomb-0.1))
       #:cargo-development-inputs
       (("rust-ntest-test-cases" ,rust-ntest-test-cases-0.7)
        ("rust-ntest-timeout" ,rust-ntest-timeout-0.7)
        ("rust-timebomb" ,rust-timebomb-0.1))))
    (home-page "https://github.com/becheran/ntest")
    (synopsis "Testing framework for Rust")
    (description "This package provides a testing framework for Rust which
enhances the built-in library with some useful features.")
    (license license:expat)))

(define-public rust-ntest-proc-macro-helper-0.7
  (package
    (name "rust-ntest-proc-macro-helper")
    (version "0.7.1")
    (source
     (origin
       (method url-fetch)
       (uri (crate-uri "ntest-proc-macro-helper" version))
       (file-name (string-append name "-" version ".tar.gz"))
       (sha256
        (base32
         "0xkyp5yc91bc3bc8441sjgdia3l6g7nclhy53nq5zw2fk3q6ww3y"))))
    (build-system cargo-build-system)
    (home-page "https://github.com/becheran/ntest")
    (synopsis "Testing framework for Rust")
    (description "This package provides a testing framework for Rust which
enhances the built-in library with some useful features.")
    (license license:expat)))

(define-public rust-ntest-test-cases-0.7
  (package
    (name "rust-ntest-test-cases")
    (version "0.7.1")
    (source
     (origin
       (method url-fetch)
       (uri (crate-uri "ntest_test_cases" version))
       (file-name (string-append name "-" version ".tar.gz"))
       (sha256
        (base32
         "0d1h1w33sznc1y3l0ka5xa1mpznn8qqxjv3s5qi9y2fs7091fcvz"))))
    (build-system cargo-build-system)
    (arguments
     `(#:cargo-inputs
       (("rust-proc-macro2" ,rust-proc-macro2-1)
        ("rust-quote" ,rust-quote-1)
        ("rust-syn" ,rust-syn-1))))
    (home-page "https://github.com/becheran/ntest")
    (synopsis "Test cases for ntest framework")
    (description "This package provides test cases for ntest framework.")
    (license license:expat)))

(define-public rust-ntest-timeout-0.7
  (package
    (name "rust-ntest-timeout")
    (version "0.7.1")
    (source
     (origin
       (method url-fetch)
       (uri (crate-uri "ntest_timeout" version))
       (file-name (string-append name "-" version ".tar.gz"))
       (sha256
        (base32
         "026isn882xadp9c0pcfzhiz01zj7qn31mkmgcbmy8x3gd4dqx7rr"))))
    (build-system cargo-build-system)
    (arguments
     `(#:cargo-inputs
       (("rust-proc-macro2" ,rust-proc-macro2-1)
        ("rust-quote" ,rust-quote-1)
        ("rust-syn" ,rust-syn-1)
        ("rust-timebomb" ,rust-timebomb-0.1))))
    (home-page "https://github.com/becheran/ntest")
    (synopsis "Timeout attribute for the ntest framework")
    (description "This package provides a timeout attribute for the ntest
framework.")
    (license license:expat)))

(define-public rust-doc-comment-0.3
  (package
    (name "rust-doc-comment")
    (version "0.3.3")
    (source
      (origin
        (method url-fetch)
        (uri (crate-uri "doc-comment" version))
        (file-name (string-append name "-" version ".crate"))
        (sha256
         (base32
          "043sprsf3wl926zmck1bm7gw0jq50mb76lkpk49vasfr6ax1p97y"))))
    (build-system cargo-build-system)
    (arguments '(#:skip-build? #t))
    (home-page "https://github.com/GuillaumeGomez/doc-comment")
    (synopsis "Macro to generate doc comments")
    (description "This package provides a way to generate doc comments
from macros.")
    (license license:expat)))

(define-public rust-zbus-macros-1
  (package
    (name "rust-zbus-macros")
    (version "1.8.0")
    (source
     (origin
       (method url-fetch)
       (uri (crate-uri "zbus-macros" version))
       (file-name (string-append name "-" version ".tar.gz"))
       (sha256
        (base32 "0c093vxdly5v7hy8z7i9z5l3j79ris0g3bcv6pplnf3jv8f19k47"))))
    (build-system cargo-build-system)
    (arguments
     `(#:cargo-inputs
       (("rust-proc-macro2" ,rust-proc-macro2-1)
        ("rust-syn" ,rust-syn-1)
        ("rust-proc-macro-crate" ,rust-proc-macro-crate-0.1))
       #:cargo-development-inputs
       (("rust-zvariant" ,rust-zvariant-2)
        ("rust-zbus" ,rust-zbus-1)
        ("rust-serde" ,rust-serde-1)
        ("rust-trybuild" ,rust-trybuild-1))))
    (home-page "https://gitlab.freedesktop.org/levans/zbus")
    (synopsis "A Rust API for D-Bus communication.")
    (description
     "A Rust API for D-Bus communication. The aim is to provide a safe and simple high- and low-level API akin to
GDBus, that doesn't depend on C libraries.")
    (license license:expat)))

(define-public rust-zbus-polkit-1
  (package
    (name "rust-zbus-polkit")
    (version "1.0.1")
    (source
     (origin
       (method url-fetch)
       (uri (crate-uri "zbus-polkit" version))
       (file-name (string-append name "-" version ".tar.gz"))
       (sha256
        (base32 "09nvf1mgdpbj27cnwbjrj1pajmxbp11gh4j4s3k0p7iaa23ab1ns"))))
    (build-system cargo-build-system)
    (arguments
     `(#:cargo-inputs
       (("rust-zvariant" ,rust-zvariant-2)
        ("rust-zbus" ,rust-zbus-1)
        ("rust-serde-repr" ,rust-serde-repr-0.1)
        ("rust-enumflags2" ,rust-enumflags2-0.6))))
    (home-page "https://gitlab.freedesktop.org/levans/zbus")
    (synopsis "A Rust API for D-Bus communication.")
    (description
     "A Rust API for D-Bus communication. The aim is to provide a safe and simple high- and low-level API akin to
GDBus, that doesn't depend on C libraries.")
    (license license:expat)))

(define-public rust-zbus-1
  (package
    (name "rust-zbus")
    (version "1.8.0")
    (source
     (origin
       (method url-fetch)
       (uri (crate-uri "zbus" version))
       (file-name (string-append name "-" version ".tar.gz"))
       (sha256
        (base32 "1a023ykgvavp28p5zzgxl14lj639p4vhqpypmqr4xvfs76md9d20"))))
    (build-system cargo-build-system)
    (arguments
     `(#:tests? #false
       #:cargo-inputs
       (("rust-byteorder" ,rust-byteorder-1)
        ("rust-nix" ,rust-nix-0.17)
        ("rust-serde" ,rust-serde-1)
        ("rust-serde-repr" ,rust-serde-repr-0.1)
        ("rust-serde-xml-rs" ,rust-serde-xml-rs-0.4)
        ("rust-zvariant" ,rust-zvariant-2)
        ("rust-zbus-macros" ,rust-zbus-macros-1)
        ("rust-enumflags2" ,rust-enumflags2-0.6)
        ("rust-derivative" ,rust-derivative-2)
        ("rust-scoped-tls" ,rust-scoped-tls-1)
        ("rust-fastrand" ,rust-fastrand-1)
        ("rust-once-cell" ,rust-once-cell-1)
        ("rust-async-io" ,rust-async-io-1)
        ("rust-proc-macro-crate" ,rust-proc-macro-crate-0.1))
       #:cargo-development-inputs
       (("rust-zbus-polkit" ,rust-zbus-polkit-1)
        ("rust-doc-comment" ,rust-doc-comment-0.3)
        ("rust-ntest" ,rust-ntest-0.7))))
    (home-page "https://gitlab.freedesktop.org/levans/zbus")
    (synopsis "A Rust API for D-Bus communication.")
    (description
     "A Rust API for D-Bus communication. The aim is to provide a safe and simple high- and low-level API akin to
GDBus, that doesn't depend on C libraries.")
    (license license:expat)))

(define-public rust-gtk-0.9
  (package
    (name "rust-gtk")
    (version "0.9.0")
    (source
     (origin
       (method url-fetch)
       (uri (crate-uri "gtk" version))
       (file-name
        (string-append name "-" version ".tar.gz"))
       (sha256
        (base32 "17gab15byxfhmzq2ax7nd6aay611l7gjxqysykjdb1m6gggnmprs"))))
    (build-system cargo-build-system)
    (arguments
     `(#:cargo-inputs
       (("rust-atk" ,rust-atk-0.9)
        ("rust-bitflags" ,rust-bitflags-1)
        ("rust-cairo-rs" ,rust-cairo-rs-0.9)
        ("rust-cairo-sys-rs" ,rust-cairo-sys-rs-0.9)
        ("rust-cc" ,rust-cc-1)
        ("rust-gdk" ,rust-gdk-0.13)
        ("rust-gdk-pixbuf" ,rust-gdk-pixbuf-0.8)
        ("rust-gdk-pixbuf-sys" ,rust-gdk-pixbuf-sys-0.9)
        ("rust-gdk-sys" ,rust-gdk-sys-0.9)
        ("rust-gio" ,rust-gio-0.8)
        ("rust-gio-sys" ,rust-gio-sys-0.9)
        ("rust-glib" ,rust-glib-0.9)
        ("rust-glib-sys" ,rust-glib-sys-0.9)
        ("rust-gobject-sys" ,rust-gobject-sys-0.9)
        ("rust-gtk-rs-lgpl-docs" ,rust-gtk-rs-lgpl-docs-0.1)
        ("rust-gtk-sys" ,rust-gtk-sys-0.10)
        ("rust-lazy-static" ,rust-lazy-static-1)
        ("rust-libc" ,rust-libc-0.2)
        ("rust-pango" ,rust-pango-0.8)
        ("rust-pango-sys" ,rust-pango-sys-0.9))
       #:cargo-development-inputs
       (("rust-gir-format-check" ,rust-gir-format-check-0.1))))
    (inputs
     `(("atk" ,atk)
       ("cairo" ,cairo)
       ("glib" ,glib)
       ("gtk+" ,gtk+)
       ("pango" ,pango)))
    (home-page "https://gtk-rs.org/")
    (synopsis "Rust bindings for the GTK+ 3 library")
    (description "This package provides Rust bindings for the GTK+ 3 library.")
    (license license:expat)))

(define-public rust-atk-0.9
  (package
    (name "rust-atk")
    (version "0.9.0")
    (source
     (origin
       (method url-fetch)
       (uri (crate-uri "atk" version))
       (file-name (string-append name "-" version ".tar.gz"))
       (sha256
        (base32 "09n46zp8jgxspdzhmi93cag79jjnr0ila94n8nr53g8hw88ljaw1"))))
    (build-system cargo-build-system)
    (arguments
     `(#:cargo-inputs
       (("rust-atk-sys" ,rust-atk-sys-0.10)
        ("rust-bitflags" ,rust-bitflags-1)
        ("rust-glib" ,rust-glib-0.9)
        ("rust-glib-sys" ,rust-glib-sys-0.9)
        ("rust-gobject-sys" ,rust-gobject-sys-0.9)
        ("rust-gtk-rs-lgpl-docs" ,rust-gtk-rs-lgpl-docs-0.1)
        ("rust-libc" ,rust-libc-0.2))
       #:cargo-development-inputs
       (("rust-gir-format-check" ,rust-gir-format-check-0.1))))
    (inputs
     `(("atk" ,atk)
       ("glib" ,glib)))
    (home-page "https://gtk-rs.org/")
    (synopsis "Rust bindings for the ATK library")
    (description "Rust bindings for the ATK library")
    (license license:expat)))

(define-public rust-gtk-sys-0.10
  (package
    (name "rust-gtk-sys")
    (version "0.10.0")
    (source
     (origin
       (method url-fetch)
       (uri (crate-uri "gtk-sys" version))
       (file-name (string-append name "-" version ".tar.gz"))
       (sha256
        (base32 "0mq4i161kk6dwiz19ayxgm9fhx7n3r5lm9lbjiyk0qs811pxmb49"))))
    (build-system cargo-build-system)
    (arguments
     `(#:tests? #f                      ;missing files
       #:cargo-inputs
       (("rust-atk-sys" ,rust-atk-sys-0.10)
        ("rust-cairo-sys-rs" ,rust-cairo-sys-rs-0.9)
        ("rust-gdk-pixbuf-sys" ,rust-gdk-pixbuf-sys-0.9)
        ("rust-gdk-sys" ,rust-gdk-sys-0.9)
        ("rust-gio-sys" ,rust-gio-sys-0.9)
        ("rust-glib-sys" ,rust-glib-sys-0.9)
        ("rust-gobject-sys" ,rust-gobject-sys-0.9)
        ("rust-libc" ,rust-libc-0.2)
        ("rust-pango-sys" ,rust-pango-sys-0.9)
        ("rust-pkg-config" ,rust-pkg-config-0.3))
       #:cargo-development-inputs
       (("rust-shell-words" ,rust-shell-words-0.1)
        ("rust-tempfile" ,rust-tempfile-3))))
    (inputs
     `(("gtk+" ,gtk+)))
    (home-page "https://gtk-rs.org/")
    (synopsis "FFI bindings to libgtk-3")
    (description "This package provides FFI bindings to libgtk-3.")
    (license license:expat)))

(define-public rust-gtk-sys-0.10
  (package
    (name "rust-gtk-sys")
    (version "0.10.0")
    (source
     (origin
       (method url-fetch)
       (uri (crate-uri "gtk-sys" version))
       (file-name (string-append name "-" version ".tar.gz"))
       (sha256
        (base32 "0mq4i161kk6dwiz19ayxgm9fhx7n3r5lm9lbjiyk0qs811pxmb49"))))
    (build-system cargo-build-system)
    (arguments
     `(#:tests? #f                      ;missing files
       #:cargo-inputs
       (("rust-atk-sys" ,rust-atk-sys-0.10)
        ("rust-cairo-sys-rs" ,rust-cairo-sys-rs-0.9)
        ("rust-gdk-pixbuf-sys" ,rust-gdk-pixbuf-sys-0.9)
        ("rust-gdk-sys" ,rust-gdk-sys-0.9)
        ("rust-gio-sys" ,rust-gio-sys-0.9)
        ("rust-glib-sys" ,rust-glib-sys-0.9)
        ("rust-gobject-sys" ,rust-gobject-sys-0.9)
        ("rust-libc" ,rust-libc-0.2)
        ("rust-pango-sys" ,rust-pango-sys-0.9)
        ("rust-pkg-config" ,rust-pkg-config-0.3))
       #:cargo-development-inputs
       (("rust-shell-words" ,rust-shell-words-0.1)
        ("rust-tempfile" ,rust-tempfile-3))))
    (inputs
     `(("gtk+" ,gtk+)))
    (home-page "https://gtk-rs.org/")
    (synopsis "FFI bindings to libgtk-3")
    (description "This package provides FFI bindings to libgtk-3.")
    (license license:expat)))

(define-public rust-atk-sys-0.10
  (package
    (name "rust-atk-sys")
    (version "0.10.0")
    (source
     (origin
       (method url-fetch)
       (uri (crate-uri "atk-sys" version))
       (file-name (string-append name "-" version ".tar.gz"))
       (sha256
        (base32 "1knzvq2jdkx1nav619jbqsx2ivzh901rsp2wl57wr50x2fpy8c7m"))))
    (build-system cargo-build-system)
    (arguments
     `(#:tests? #f                      ;missing files
       #:cargo-inputs
       (("rust-glib-sys" ,rust-glib-sys-0.9)
        ("rust-gobject-sys" ,rust-gobject-sys-0.9)
        ("rust-libc" ,rust-libc-0.2)
        ("rust-pkg-config" ,rust-pkg-config-0.3))
       #:cargo-development-inputs
       (("rust-shell-words" ,rust-shell-words-0.1)
        ("rust-tempfile" ,rust-tempfile-3))))
    (inputs
     `(("atk" ,atk)
       ("glib" ,glib)))
    (home-page "https://gtk-rs.org/")
    (synopsis "FFI bindings to libatk-1")
    (description "FFI bindings to libatk-1")
    (license license:expat)))

(define-public rust-gtk-0.9
  (package
    (name "rust-gtk")
    (version "0.9.0")
    (source
     (origin
       (method url-fetch)
       (uri (crate-uri "gtk" version))
       (file-name
        (string-append name "-" version ".tar.gz"))
       (sha256
        (base32 "17gab15byxfhmzq2ax7nd6aay611l7gjxqysykjdb1m6gggnmprs"))))
    (build-system cargo-build-system)
    (arguments
     `(#:cargo-inputs
       (("rust-atk" ,rust-atk-0.9)
        ("rust-bitflags" ,rust-bitflags-1)
        ("rust-cairo-rs" ,rust-cairo-rs-0.9)
        ("rust-cairo-sys-rs" ,rust-cairo-sys-rs-0.9)
        ("rust-cc" ,rust-cc-1)
        ("rust-gdk" ,rust-gdk-0.13)
        ("rust-gdk-pixbuf" ,rust-gdk-pixbuf-0.8)
        ("rust-gdk-pixbuf-sys" ,rust-gdk-pixbuf-sys-0.9)
        ("rust-gdk-sys" ,rust-gdk-sys-0.9)
        ("rust-gio" ,rust-gio-0.8)
        ("rust-gio-sys" ,rust-gio-sys-0.9)
        ("rust-glib" ,rust-glib-0.9)
        ("rust-glib-sys" ,rust-glib-sys-0.9)
        ("rust-gobject-sys" ,rust-gobject-sys-0.9)
        ("rust-gtk-rs-lgpl-docs" ,rust-gtk-rs-lgpl-docs-0.1)
        ("rust-gtk-sys" ,rust-gtk-sys-0.10)
        ("rust-lazy-static" ,rust-lazy-static-1)
        ("rust-libc" ,rust-libc-0.2)
        ("rust-pango" ,rust-pango-0.8)
        ("rust-pango-sys" ,rust-pango-sys-0.9))
       #:cargo-development-inputs
       (("rust-gir-format-check" ,rust-gir-format-check-0.1))))
    (inputs
     `(("atk" ,atk)
       ("cairo" ,cairo)
       ("glib" ,glib)
       ("gtk+" ,gtk+)
       ("pango" ,pango)))
    (home-page "https://gtk-rs.org/")
    (synopsis "Rust bindings for the GTK+ 3 library")
    (description "This package provides Rust bindings for the GTK+ 3 library.")
    (license license:expat)))

(define-public rust-libappindicator-sys-0.5
  (package
    (name "rust-libappindicator-sys")
    (version "0.5.0")
    (source
     (origin
       (method url-fetch)
       (uri (crate-uri "libappindicator-sys" version))
       (file-name (string-append name "-" version ".tar.gz"))
       (sha256
        (base32
         "07b5r3wy5kq4x3zsmgbgmz2645yr5fyy1finkbsj3cbc38hbl6cp"))))
    (build-system cargo-build-system)
    (arguments
     `(#:cargo-inputs
       (("rust-gtk-sys" ,rust-gtk-sys-0.10)
        ("rust-bindgen" ,rust-bindgen-0.52)
        ("rust-pkg-config" ,rust-pkg-config-0.3))))
    (home-page "https://github.com/qdot/libappindicator-sys")
    (synopsis "Bindings for the libappindicator library.")
    (description "Bindings for the libappindicator library. libappindicator provides 
cross-distribution/window system functions for creating systray icons and menus.")
    (license license:lgpl3+)))

(define-public rust-libappindicator-0.5
  (package
    (name "rust-libappindicator")
    (version "0.5.2")
    (source
     (origin
       (method url-fetch)
       (uri (crate-uri "libappindicator" version))
       (file-name (string-append name "-" version ".tar.gz"))
       (sha256
        (base32
         "0dyhb5gh2srhaq8v6nrcds4qsqibkpgkh53lypxf4l1z6mbzrlsj"))))
    (build-system cargo-build-system)
    (arguments
     `(#:cargo-inputs
       (("rust-gtk" ,rust-gtk-0.9)
        ("rust-glib" ,rust-glib-0.10)
        ("rust-log" ,rust-log-0.4)
        ("rust-gtk-sys" ,rust-gtk-sys-0.10)
        ("rust-libappindicator-sys" ,rust-libappindicator-sys-0.5))))
    (home-page "https://github.com/qdot/libappindicator-rs")
    (synopsis "Bindings for the libappindicator library.")
    (description "Bindings for the libappindicator library. libappindicator provides 
cross-distribution/window system functions for creating systray icons and menus.")
    (license license:lgpl3+)))

(define-public rust-dlv-list-0.2
  (package
    (name "rus-dlv-list")
    (version "0.2.2")
    (source
     (origin
       (method url-fetch)
       (uri (crate-uri "dlv-list" version))
       (file-name (string-append name "-" version ".tar.gz"))
       (sha256
        (base32
         "0v42n28mx9r6vr43dsqkh5dgvb9m0wyjp7fb20m331m7p48ijf8v"))))
    (build-system cargo-build-system)
    (arguments
     `(#:cargo-inputs
       ;;unable to build rust-rand-0.7
       ;;(("rust-rand" ,rust-rand-0.7))))
       (("rust-rand" ,rust-my-rand-0.7))))
    (home-page "https://github.com/sgodwincs/dlv-list-rs")
    (synopsis "Semi-doubly linked list implemented using a vector.")
    (description "Semi-doubly linked list implemented using a vector.")
    (license license:expat)))

(define-public rust-ordered-multimap-0.3
  (package
    (name "rust-ordered-multimap")
    (version "0.3.1")
    (source
     (origin
       (method url-fetch)
       (uri (crate-uri "ordered-multimap" version))
       (file-name (string-append name "-" version ".tar.gz"))
       (sha256
        (base32
         "1194q7sb2d6chbllsn7237dhhvx04iqr3sq0ii16w1pcv5x2qrqw"))))
    (build-system cargo-build-system)
    (arguments
     `(#:cargo-inputs
       (("rust-dlv-list" ,rust-dlv-list-0.2)
        ("rust-hashbrown" ,rust-hashbrown-0.9)
        ("rust-serde" ,rust-serde-1))))
    (home-page "https://github.com/sgodwincs/ordered-multimap-rs")
    (synopsis "This is a multimap meaning that multiple values can be associated with a given key")
    (description "Currently, this crate contains a single type ListOrderedMultimap. This is a multimap
 meaning that multiple values can be associated with a given key,
 but it also maintains insertion order across all keys and values.")
    (license license:expat)))

;;unable to build rust-gdk-0.13 from crates-gtk.scm
(define-public rust-my-gdk-0.13
  (package
    (name "rust-gdk")
    (version "0.13.2")
    (source
     (origin
       (method url-fetch)
       (uri (crate-uri "gdk" version))
       (file-name (string-append name "-" version ".tar.gz"))
       (sha256
        (base32 "0zbb9bwg2z9vlcjj9b59qch3mfmszsrxya7syc5a39v85adq606v"))))
    (build-system cargo-build-system)
    (arguments
     `(#:skip-build? #t
       #:cargo-inputs
       (("rust-bitflags" ,rust-bitflags-1)
        ;;unable to build rust-cairo-rs-0.9
        ;;("rust-cairo-rs" ,rust-cairo-rs-0.9)
        ("rust-cairo-rs" ,rust-my-cairo-rs-0.9)
        ("rust-cairo-sys-rs" ,rust-cairo-sys-rs-0.10)
        ("rust-gdk-pixbuf" ,rust-gdk-pixbuf-0.9)
        ("rust-gdk-sys" ,rust-gdk-sys-0.10)
        ("rust-gio" ,rust-gio-0.9)
        ("rust-gio-sys" ,rust-gio-sys-0.10)
        ("rust-glib" ,rust-glib-0.10)
        ("rust-glib-sys" ,rust-glib-sys-0.10)
        ("rust-gobject-sys" ,rust-gobject-sys-0.10)
        ("rust-gtk-rs-lgpl-docs" ,rust-gtk-rs-lgpl-docs-0.1)
        ("rust-libc" ,rust-libc-0.2)
        ("rust-pango" ,rust-pango-0.9))
       #:cargo-development-inputs
       (("rust-gir-format-check" ,rust-gir-format-check-0.1))))
    (inputs
     `(("cairo" ,cairo)
       ("gdk-pixbuf" ,gdk-pixbuf)
       ("glib" ,glib)
       ("gtk+" ,gtk+)
       ("pango" ,pango)))
    (home-page "https://gtk-rs.org/")
    (synopsis "Rust bindings for the GDK 3 library")
    (description "This package provides Rust bindings for the GDK 3 library.")
    (license license:expat)))

;;unable to build rust-cairo-rs-0.9
(define-public rust-my-cairo-rs-0.9
  (package
    (name "rust-cairo-rs")
    (version "0.9.1")
    (source
     (origin
       (method url-fetch)
       (uri (crate-uri "cairo-rs" version))
       (file-name (string-append name "-" version ".tar.gz"))
       (sha256
        (base32 "1f5x6ipfpzz0ffph0pg0xfkdfcbr0jp59714zz857jp88zhg5h65"))))
    (build-system cargo-build-system)
    (arguments
     `(#:skip-build? #t
       #:cargo-inputs
       (("rust-bitflags" ,rust-bitflags-1)
        ;;unable to build rust-cairo-sys-rs-0.10
        ;;("rust-cairo-sys-rs" ,rust-cairo-sys-rs-0.10)
        ("rust-cairo-sys-rs" ,rust-my-cairo-sys-rs-0.10)
        ("rust-glib" ,rust-glib-0.10)
        ("rust-glib-sys" ,rust-glib-sys-0.10)
        ("rust-gobject-sys" ,rust-gobject-sys-0.10)
        ("rust-gtk-rs-lgpl-docs" ,rust-gtk-rs-lgpl-docs-0.1)
        ("rust-libc" ,rust-libc-0.2)
        ("rust-thiserror" ,rust-thiserror-1))
       #:cargo-development-inputs
       (("rust-tempfile" ,rust-tempfile-3))))
    (inputs
     `(("cairo" ,cairo)))
    (home-page "https://gtk-rs.org/")
    (synopsis "Rust bindings for the Cairo library")
    (description "Rust bindings for the Cairo library")
    (license license:expat)))

;;unable to build rust-cairo-sys-rs-0.10
(define-public rust-my-cairo-sys-rs-0.10
  (package
    (name "rust-cairo-sys-rs")
    (version "0.10.0")
    (source
     (origin
       (method url-fetch)
       (uri (crate-uri "cairo-sys-rs" version))
       (file-name (string-append name "-" version ".tar.gz"))
       (sha256
        (base32 "19wch8zc11hbi724mn16hhqyff8kw5c5bsbdlzpxdwfmkadn7lif"))))
    (build-system cargo-build-system)
    (arguments
     `(#:skip-build? #t
       #:cargo-inputs
       (("rust-glib-sys" ,rust-glib-sys-0.10)
        ("rust-libc" ,rust-libc-0.2)
        ;;unable to build rust-system-deps-1
        ;;("rust-system-deps" ,rust-system-deps-1)
        ("rust-system-deps" ,rust-my-system-deps-1)
        ("rust-winapi" ,rust-winapi-0.3)
        ("rust-x11" ,rust-x11-2))))
    (inputs
     `(("cairo" ,cairo)))
    (home-page "https://gtk-rs.org/")
    (synopsis "FFI bindings to libcairo")
    (description "This package provides FFI bindings to libcairo.")
    (license license:expat)))

;;unable to build rust-system-deps-1
(define-public rust-my-system-deps-1
  (package
    (name "rust-system-deps")
    (version "1.3.2")
    (source
     (origin
       (method url-fetch)
       (uri (crate-uri "system-deps" version))
       (file-name (string-append name "-" version ".tar.gz"))
       (sha256
        (base32 "16v4ljmj8sj030mdcc1yk615vciqlyxi7csq6lxka6cs4qbwqghg"))))
    (build-system cargo-build-system)
    (arguments
     `(#:tests? #f                      ;source is missing some test files
       #:cargo-inputs
       (("rust-heck" ,rust-heck-0.3)
        ("rust-pkg-config" ,rust-pkg-config-0.3)
        ("rust-strum" ,rust-strum-0.18)
        ("rust-strum-macros" ,rust-strum-macros-0.18)
        ("rust-thiserror" ,rust-thiserror-1)
        ("rust-toml" ,rust-toml-0.5)
        ;;unable to build rust-version-compare-0.0
        ;;("rust-version-compare" ,rust-version-compare-0.0))
        ("rust-version-compare" ,rust-my-version-compare-0.0))
       #:cargo-development-inputs
       (("rust-itertools" ,rust-itertools-0.9))))
    (home-page "https://github.com/gdesmott/system-deps")
    (synopsis "Define system dependencies in @file{Cargo.toml}")
    (description
     "This crate lets you write system dependencies in @file{Cargo.toml}
metadata, rather than programmatically in @file{build.rs}.  This makes those
dependencies declarative, so other tools can read them as well.")
    (license (list license:expat license:asl2.0))))

;;unable to build rust-version-compare-0.0
(define-public rust-my-version-compare-0.0
  (package
    (name "rust-version-compare")
    (version "0.0.10")
    (source
     (origin
       (method url-fetch)
       (uri (crate-uri "version-compare" version))
       (file-name
        (string-append name "-" version ".tar.gz"))
       (sha256
        (base32 "18ack6rx18rp700h1dncljmpzchs3p2dfh76a8ds6vmfbfi5cdfn"))))
    (build-system cargo-build-system)
    (home-page "https://github.com/timvisee/version-compare")
    (synopsis "Rust library to easily compare version numbers")
    (description
     "This package provides a Rust library to easily compare version
numbers, and test them against various comparison operators.")
    (license license:expat)))

;;unable to build rust-rand-0.7
(define-public rust-my-rand-0.7
  (package
    (name "rust-rand")
    (version "0.7.3")
    (source
     (origin
       (method url-fetch)
       (uri (crate-uri "rand" version))
       (file-name
        (string-append name "-" version ".tar.gz"))
       (sha256
        (base32
         "00sdaimkbz491qgi6qxkv582yivl32m2jd401kzbn94vsiwicsva"))))
    (build-system cargo-build-system)
    (arguments
     `(#:cargo-inputs
       (("rust-getrandom" ,rust-getrandom-0.1)
        ("rust-libc" ,rust-libc-0.2)
        ("rust-log" ,rust-log-0.4)
        ("rust-packed-simd" ,rust-packed-simd-0.3)
        ("rust-rand-chacha" ,rust-rand-chacha-0.2)
        ("rust-rand-core" ,rust-rand-core-0.5)
        ("rust-rand-hc" ,rust-rand-hc-0.2)
        ("rust-rand-pcg" ,rust-rand-pcg-0.2))
       #:cargo-development-inputs
       (("rust-rand-hc" ,rust-rand-hc-0.2)
        ("rust-rand-pcg" ,rust-rand-pcg-0.2))))
    (home-page "https://crates.io/crates/rand")
    (synopsis "Random number generators and other randomness functionality")
    (description
     "Rand provides utilities to generate random numbers, to convert them to
useful types and distributions, and some randomness-related algorithms.")
    (license (list license:expat license:asl2.0))))

(define-public rust-gettext-rs-0.7
  (package
    (name "rust-gettext-rs")
    (version "0.7.0")
    (source
      (origin
        (method url-fetch)
        (uri (crate-uri "gettext-rs" version))
        (file-name
         (string-append name "-" version ".tar.gz"))
        (sha256
         (base32
          "0r7kahqcjrkm83d3gzzkn83fnw2bnqj2ank5z6hsm66izalai7p4"))))
    (build-system cargo-build-system)
    (arguments
     `(#:cargo-inputs
       (("rust-gettext-sys" ,rust-gettext-sys-0.21)
        ("rust-locale-config" ,rust-locale-config-0.3))))
    (inputs
     `(("gettext" ,gettext-minimal)))
    (home-page "https://github.com/Koka/gettext-rs")
    (synopsis "GNU Gettext FFI binding for Rust")
    (description "This package provides GNU Gettext FFI bindings for Rust.")
    (license license:expat)))

(define-public rust-gettext-sys-0.21
  (package
    (name "rust-gettext-sys")
    (version "0.21.0")
    (source
      (origin
        (method url-fetch)
        (uri (crate-uri "gettext-sys" version))
        (file-name
         (string-append name "-" version ".tar.gz"))
        (sha256
         (base32
          "105d5zh67yc5vyzmqxdw7hx82h606ca6rzhsfjgzjczn2s012pc8"))))
    (build-system cargo-build-system)
    (arguments
     `(#:cargo-inputs
       (("rust-cc" ,rust-cc-1)
        ("rust-tempfile" ,rust-tempfile-3))))
    (inputs
     `(("gettext" ,gettext-minimal)))
    (home-page "https://github.com/Koka/gettext-rs")
    (synopsis "Gettext raw FFI bindings")
    (description "This package provides raw FFI bindings for GNU Gettext.")
    (license license:expat)))

(define-public blue-recorder
 (let ((version "1.3.2")
       (commit "e160bfcceb58d66a3a1c075b158bf9c1a5bcf0d3"))
  (package
   (name "blue-recorder")
   (version (git-version version "+4commits" commit))
   (source (origin
            (method git-fetch)
            (uri (git-reference
                  (url "https://gitlab.com/gitlab_ly/blue-recorder/")
                  (commit commit)
                  (recursive? #t)))
            (file-name (git-file-name name version))
            (sha256
             (base32
              "0vchxxxgawh6zgh91l6dapx6a6gyfwz7vhwzlnh6nq6734rjhjxa"))))
    (build-system cargo-build-system)
    (arguments
     `(#:cargo-inputs
       (("rust-chrono" ,rust-chrono-0.4)
        ("rust-regex" ,rust-regex-1)
        ("rust-libappindicator" ,rust-libappindicator-0.5)
        ("rust-rust-ini" ,rust-rust-ini-0.16)
        ("rust-subprocess" ,rust-subprocess-0.2)
        ;;("rust-gdk" ,rust-gdk-0.13)
        ;;unable to build rust-gdk-0.13 from crates-gtk.scm
        ("rust-gdk" ,rust-my-gdk-0.13)
        ("rust-gdk-pixbuf" ,rust-gdk-pixbuf-0.9)
        ("rust-gettext-rs" ,rust-gettext-rs-0.7)
        ("rust-gio" ,rust-gio-0.9)
        ("rust-glib" ,rust-glib-0.10)
        ("rust-gtk" ,rust-gtk-0.9)
        ("rust-zbus" ,rust-zbus-1)
        ("rust-zvariant" ,rust-zvariant-2))
       #:phases
       (modify-phases %standard-phases
         (replace 'install
           (lambda* (#:key inputs outputs #:allow-other-keys)
             (let* ((out   (assoc-ref outputs "out"))
                    (bin   (string-append out "/bin"))
                    (share (string-append out "/share"))
                    (icons (string-append share "/icons/hicolor/scalable/apps"))
                    (blue-recorder "target/release/blue-recorder"))
               ;; Install the executable.
               (install-file blue-recorder bin)
               ;; Install desktop file.
               (install-file "data/blue-recorder.desktop"
                             (string-append share "/applications"))
               ;; Install interfaces file.
               (install-file "interfaces/main.ui"
                             (string-append bin "/interfaces"))
               ;; Install icon.
               (mkdir-p icons)
               (copy-file "data/blue-recorder.svg"
                          (string-append icons "/blue-recorder.svg"))
               ;; Install po.
               (copy-recursively "po"
                          (string-append bin "/po"))
               ;; Install data.
               (copy-recursively "data"
                          (string-append bin "/data"))
               #t))))))
    (native-inputs
     `(("clang" ,clang)
       ("gettext" ,gettext-minimal)
       ("pkg-config" ,pkg-config)))
    (inputs
     `(("ffmpeg" ,ffmpeg)
       ("libappindicator" ,libappindicator)
       ("gtk+" ,gtk+)
       ("xwininfo" ,xwininfo)
       ("pulseaudio" ,pulseaudio)))
   (synopsis "A simple screen recorder for Linux desktop.")
   (description "A simple desktop recorder for Linux systems. Built using Rust, GTK+ 3 and ffmpeg. It supports
 recording audio and video on almost all Linux interfaces with support for Wayland display server on GNOME session.")
   (home-page "https://github.com/xlmnxp/blue-recorder")
   (license license:gpl3+))))

blue-recorder
