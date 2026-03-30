# Proteus — Modern BusyBox Alternative in Rust

## Product Requirements Document (PRD)

**Project Name:** Proteus
**Version:** 1.0
**Language:** Rust
**License:** MIT OR Apache-2.0 (dual license)
**Mythology:** Proteus — Yunan mitolojisinde sürekli form değiştiren deniz tanrısı. Tek bir varlık, sonsuz biçim. Tek binary, yüzlerce araç.
**Tagline:** *"Shape-shifting Unix toolkit — one binary, every tool."*

---

## 1. Executive Summary

Proteus, BusyBox'ın Rust ile sıfırdan yazılmış modern alternatifidir. Tek bir statik binary içinde 100+ Unix/Linux aracını barındırır. BusyBox'ın temel felsefesine (minimal, tek binary, gömülü odaklı) sadık kalırken; memory safety, tam POSIX uyumluluk, modüler build sistemi, birinci sınıf Unicode desteği ve güvenlik odaklı tasarım ile onu aşmayı hedefler.

Birincil hedef gömülü Linux sistemleri (router, IoT, SBC) ve container ortamlarıdır. İkincil hedef olarak cross-platform desteği (Linux, FreeBSD, NetBSD, OpenBSD, macOS) sağlanır.

---

## 2. Problem Statement

BusyBox, 1999'dan beri gömülü Linux dünyasının standart aracıdır. Ancak yaşlandıkça biriken sorunları var:

- **Memory safety yok:** C ile yazılmış, buffer overflow ve use-after-free riskleri mevcut. CVE listesi uzun.
- **Tutarsız POSIX uyumluluk:** Hangi applet hangi POSIX flag'lerini destekliyor belli değil. Script portability sorunları yaygın.
- **Yetersiz Unicode/locale desteği:** UTF-8 ve multibyte karakter desteği kırık veya eksik.
- **Monolitik build sistemi:** `make menuconfig` eski ve kırılgan. Applet bazında maliyet analizi zor.
- **Güvenlik izolasyonu yok:** Her applet aynı privilege seviyesinde çalışır, seccomp profili yok.
- **Test altyapısı yetersiz:** Cross-platform, cross-architecture test coverage düşük.
- **Modülerlik yok:** Tek bir applet'i güncelleme imkanı yok, tüm binary yeniden derlenmeli.

Proteus bu sorunların tamamını adresler.

---

## 3. Goals & Non-Goals

### 3.1 Goals

- Tek statik binary içinde 100+ Unix/Linux aracı sunmak
- Tam POSIX.1-2017 uyumluluk (her applet için compliance seviyesi belgelenmiş)
- GNU uzantılarının opsiyonel feature flag olarak sunulması
- Memory-safe implementasyon (Rust, `unsafe` bloğu minimum ve audit edilmiş)
- `no_std` uyumlu core, gömülü sistemlerde runtime overhead sıfır
- Statik link ile single binary (musl libc veya pure Rust)
- Her applet bağımsız cargo feature, build sırasında dahil/hariç
- Build sonrası applet bazında boyut raporu
- İlk günden UTF-8 ve temel locale desteği
- Seccomp profilleri ile applet bazında syscall kısıtlaması
- Cross-compile desteği: x86_64, aarch64, armv7, mips, mipsel, riscv64
- Kapsamlı test suite: POSIX test vektörleri + QEMU üzerinde cross-architecture CI
- BusyBox ile drop-in uyumluluk: aynı symlink mekanizması, aynı CLI arayüzleri
- Sub-1MB base binary (minimal konfigürasyon, stripped, UPX opsiyonel)

### 3.2 Non-Goals

- systemd, networkd, udevd gibi sistem daemon'larını içermek
- Paket yöneticisi olmak (apk, opkg gibi)
- Grafik arayüz veya TUI dashboard
- Windows desteği (v1.0 kapsamında değil)
- BusyBox'ın her bug'ıyla uyumluluk (bug-for-bug compatibility hedeflenmez)
- Full GNU coreutils uyumluluğu (POSIX temel, GNU opsiyonel)

---

## 4. Architecture

### 4.1 High-Level Architecture

```
proteus (single binary)
├── proteus-core          # Shared infrastructure
│   ├── args              # Unified argument parser (POSIX + GNU style)
│   ├── io                # Buffered I/O primitives
│   ├── utf8              # UTF-8 string utilities
│   ├── locale            # Minimal locale support
│   ├── path              # Path manipulation
│   ├── permissions       # Unix permission helpers
│   ├── error             # Unified error types & formatting
│   ├── glob              # Glob/pattern matching
│   ├── regex             # Minimal regex engine (POSIX BRE/ERE)
│   └── sandbox           # Seccomp/pledge helpers
│
├── proteus-applets       # Individual tool implementations
│   ├── coreutils/        # ls, cp, mv, rm, cat, echo, ...
│   ├── textutils/        # grep, sed, awk, sort, cut, tr, wc, ...
│   ├── fileutils/        # find, xargs, tar, gzip, bzip2, ...
│   ├── netutils/         # wget, nc, ping, traceroute, nslookup, ...
│   ├── procutils/        # ps, kill, top, free, uptime, ...
│   ├── shellutils/       # sh, test, expr, printf, ...
│   ├── sysutils/         # mount, umount, fdisk, mkfs, sysctl, ...
│   ├── editors/          # vi (minimal), diff, patch, ...
│   └── misc/             # date, cal, bc, hexdump, ...
│
├── proteus-shell         # POSIX sh implementation (Nereus shell)
│   ├── lexer
│   ├── parser
│   ├── interpreter
│   ├── builtins
│   ├── line_editing      # editline-compatible input
│   └── completion        # Basic tab completion
│
└── proteus-build         # Build tooling
    ├── feature_matrix    # Applet <-> feature flag mapping
    ├── size_report       # Post-build size analysis
    └── compliance_check  # POSIX compliance verification
```

### 4.2 Dispatch Mechanism

BusyBox ile aynı `argv[0]` dispatch mekanizması:

```rust
fn main() {
    let argv0 = std::env::args()
        .next()
        .and_then(|s| s.rsplit('/').next().map(String::from))
        .unwrap_or_else(|| "proteus".into());

    let exit_code = match argv0.as_str() {
        "proteus" => proteus_main(),   // Multi-call dispatch
        "ls"      => applet_ls(),
        "cat"     => applet_cat(),
        "grep"    => applet_grep(),
        // ... 100+ applet entries
        _ => {
            eprintln!("proteus: unknown applet '{}'", argv0);
            1
        }
    };

    std::process::exit(exit_code);
}
```

Ayrıca `proteus <applet> [args...]` syntax'ı da desteklenir:

```bash
proteus ls -la /tmp
proteus grep -r "pattern" .
proteus --list              # Tüm applet'leri listele
proteus --list-full         # Applet + POSIX compliance detayı
proteus <applet> --posix-info  # Spesifik applet compliance bilgisi
```

### 4.3 Symlink Installer

```bash
proteus --install [-s] <directory>
```

- `-s` (symlink): Her applet için hedef dizine symlink oluşturur
- Symlink olmadan: hardlink oluşturur
- Mevcut dosyaları overwrite etmez (`--force` ile override)
- `proteus --uninstall <directory>` ile temizleme

### 4.4 Memory Model

- Heap allocation minimuma indirilmiş
- Stack-based buffer'lar tercih edilir
- Büyük dosya işlemleri streaming/mmap ile
- `alloc` crate opsiyonel (`no_std` + `no_alloc` minimal profil mümkün)
- Global state yok, her applet kendi context'ini taşır

---

## 5. Core Library (proteus-core)

### 5.1 Argument Parser

Özel bir argument parser. `clap` gibi ağır bağımlılıklar kullanılmaz.

```rust
pub struct ArgParser {
    // POSIX short opts: -a, -b, -c
    // GNU long opts: --all, --verbose (feature flag ile)
    // Combined short: -abc
    // Option arguments: -f <file>, --file=<name>
    // -- separator for positional args
}
```

Özellikler:
- Zero-allocation parsing modu (stack buffer)
- POSIX option parsing kurallarına tam uyum
- GNU long options opsiyonel (`feature = "gnu-opts"`)
- Otomatik `--help` ve `--version` üretimi
- `--posix-info` flag'i her applet'te mevcut

### 5.2 I/O Layer

```rust
pub struct BufInput { /* ... */ }
pub struct BufOutput { /* ... */ }
```

- Configurable buffer size (varsayılan 8KB, gömülü için 512B'a düşürülebilir)
- `splice()` / `sendfile()` syscall desteği (Linux'ta zero-copy)
- Otomatik stdin/stdout/stderr detect (pipe vs terminal)
- Line-buffered vs full-buffered otomatik seçim

### 5.3 UTF-8 Engine

ICU kullanmadan minimal UTF-8 desteği:

- Codepoint iteration ve validation
- Grapheme cluster boundary detection (UAX #29 subset)
- Case mapping tabloları (Unicode 15.0 subset, ~20KB)
- Türkçe İ/I, ı/i dönüşümü dahil locale-aware case folding
- Codepoint-aware `wc`, `cut`, `sort`, `tr` desteği
- Collation: Temel Unicode collation (DUCET subset), locale-specific sıralama opsiyonel

### 5.4 Regex Engine

POSIX BRE ve ERE desteği, sıfırdan yazılmış:

- NFA tabanlı (backtracking yok, ReDoS-safe)
- POSIX BRE (Basic Regular Expression)
- POSIX ERE (Extended Regular Expression)
- Bracket expressions, back-references (BRE), character classes
- `\w`, `\d`, `\s` gibi yaygın kısayollar GNU uyumluluk flag'i ile
- UTF-8-aware character class matching

### 5.5 Sandbox Module

```rust
pub fn apply_seccomp_profile(applet: &str) -> Result<(), SandboxError>;
pub fn drop_capabilities(keep: &[Capability]) -> Result<(), SandboxError>;
```

- Her applet için önceden tanımlı seccomp-bpf profili
- `cat`: sadece read, write, open, close, fstat, mmap, exit_group
- `wget`: socket, connect, read, write, open, close + DNS resolver syscalls
- `sh`: kısıtlama yok (shell her şeyi çağırabilir)
- Linux capabilities aware: gereksiz capability'ler drop edilir
- FreeBSD'de `capsicum`, OpenBSD'de `pledge` kullanılır
- `--no-sandbox` flag'i ile devre dışı bırakılabilir

---

## 6. Shell: Nereus

Proteus'un yerleşik shell'i. İsim: Nereus — Proteus'un babası, deniz tanrılarının en bilgesi.

### 6.1 Uyumluluk

- POSIX.1-2017 Shell Command Language tam uyumluluk
- Opsiyonel bashism'ler (`feature = "shell-extended"`):
  - `[[ ]]` conditional expressions
  - Indexed arrays (associative array yok, kasıtlı)
  - `$(< file)` shorthand
  - `{start..end}` brace expansion
  - `LINENO`, `FUNCNAME` variables

### 6.2 Features

- **Line editing:** editline-uyumlu, emacs ve vi modları
- **Tab completion:** komut, dosya yolu, değişken tamamlama
- **Hata mesajları:** Okunabilir, satır numarası ve context ile
- **Syntax highlighting:** Terminal modunda opsiyonel renklendirme
- **History:** Kalıcı history dosyası, arama (`Ctrl+R`)
- **Job control:** `fg`, `bg`, `jobs`, `&`, `Ctrl+Z`
- **Startup dosyaları:** `/etc/proteus/profile`, `~/.proteusrc`

### 6.3 Non-Goals (Shell)

- Bash tam uyumluluk hedeflenmez
- Plugin sistemi yok
- Scripting dili genişletmeleri (zsh/fish tarzı) yok

---

## 7. Applet Categories & Full List

### 7.1 Coreutils (35 applet)

| Applet | POSIX | GNU Ext | Açıklama |
|--------|-------|---------|----------|
| `ls` | Tam | Opsiyonel | Dizin listeleme |
| `cp` | Tam | Opsiyonel | Dosya kopyalama |
| `mv` | Tam | Opsiyonel | Dosya taşıma/yeniden adlandırma |
| `rm` | Tam | Opsiyonel | Dosya silme |
| `cat` | Tam | Opsiyonel | Dosya birleştirme/görüntüleme |
| `echo` | Tam | — | Stdout'a yazdırma |
| `printf` | Tam | — | Formatlı çıktı |
| `head` | Tam | Opsiyonel | Dosya başını göster |
| `tail` | Tam | Opsiyonel | Dosya sonunu göster (`-f` dahil) |
| `tee` | Tam | — | stdin'i çoğalt |
| `ln` | Tam | — | Link oluşturma |
| `mkdir` | Tam | — | Dizin oluşturma |
| `rmdir` | Tam | — | Boş dizin silme |
| `touch` | Tam | — | Timestamp güncelle/dosya oluştur |
| `chmod` | Tam | — | İzin değiştirme |
| `chown` | Tam | — | Sahiplik değiştirme |
| `chgrp` | Tam | — | Grup değiştirme |
| `pwd` | Tam | — | Mevcut dizin |
| `basename` | Tam | — | Dosya adı çıkarma |
| `dirname` | Tam | — | Dizin yolu çıkarma |
| `realpath` | Tam | — | Absolute path çözümleme |
| `readlink` | Tam | — | Symlink hedefi okuma |
| `stat` | — | Evet | Dosya bilgisi |
| `mktemp` | — | Evet | Geçici dosya/dizin oluşturma |
| `sync` | Tam | — | Filesystem buffer flush |
| `true` | Tam | — | Exit 0 |
| `false` | Tam | — | Exit 1 |
| `yes` | — | Evet | Sonsuz tekrar |
| `sleep` | Tam | Opsiyonel | Bekle |
| `uname` | Tam | Opsiyonel | Sistem bilgisi |
| `id` | Tam | — | Kullanıcı/grup bilgisi |
| `whoami` | Tam | — | Mevcut kullanıcı |
| `groups` | Tam | — | Kullanıcı grupları |
| `env` | Tam | — | Ortam değişkenleri |
| `nohup` | Tam | — | HUP sinyali engelle |

### 7.2 Text Processing (20 applet)

| Applet | POSIX | GNU Ext | Açıklama |
|--------|-------|---------|----------|
| `grep` | Tam | Opsiyonel | Metin arama |
| `egrep` | Tam | — | ERE ile grep |
| `fgrep` | Tam | — | Fixed string grep |
| `sed` | Tam | Opsiyonel | Stream editor |
| `awk` | Tam | Opsiyonel | Pattern/action processor |
| `sort` | Tam | Opsiyonel | Sıralama |
| `uniq` | Tam | Opsiyonel | Tekrar filtreleme |
| `cut` | Tam | — | Alan/sütun kesme |
| `paste` | Tam | — | Satır birleştirme |
| `join` | Tam | — | Dosya birleştirme (relational) |
| `tr` | Tam | — | Karakter dönüştürme/silme |
| `wc` | Tam | — | Sayma (satır, kelime, byte, karakter) |
| `head` | Tam | Opsiyonel | İlk N satır |
| `tail` | Tam | Opsiyonel | Son N satır |
| `tac` | — | Evet | Ters satır sırası |
| `rev` | — | Evet | Satır içi ters çevirme |
| `fold` | Tam | — | Satır katlama |
| `expand` | Tam | — | Tab → space |
| `unexpand` | Tam | — | Space → tab |
| `comm` | Tam | — | İki dosya karşılaştırma |

### 7.3 File Utilities (18 applet)

| Applet | POSIX | GNU Ext | Açıklama |
|--------|-------|---------|----------|
| `find` | Tam | Opsiyonel | Dosya arama |
| `xargs` | Tam | — | stdin'den komut oluşturma |
| `tar` | Tam | Opsiyonel | Arşivleme |
| `gzip` | — | Evet | Gzip sıkıştırma/açma |
| `gunzip` | — | Evet | Gzip açma |
| `bzip2` | — | Evet | Bzip2 sıkıştırma |
| `bunzip2` | — | Evet | Bzip2 açma |
| `xz` | — | Evet | XZ sıkıştırma |
| `unxz` | — | Evet | XZ açma |
| `zcat` | — | Evet | Sıkıştırılmış dosya okuma |
| `cpio` | Tam | — | Arşiv kopyalama |
| `dd` | Tam | Opsiyonel | Blok kopyalama |
| `install` | — | Evet | Dosya kurulumu |
| `file` | — | Evet | Dosya tipi tespiti (magic bytes) |
| `md5sum` | — | Evet | MD5 checksum |
| `sha256sum` | — | Evet | SHA-256 checksum |
| `sha512sum` | — | Evet | SHA-512 checksum |
| `cmp` | Tam | — | Byte-level karşılaştırma |

### 7.4 Network Utilities (12 applet)

| Applet | POSIX | Açıklama |
|--------|-------|----------|
| `wget` | — | HTTP/HTTPS downloader (TLS: rustls) |
| `nc` | — | Netcat (TCP/UDP) |
| `ping` | — | ICMP ping |
| `ping6` | — | ICMPv6 ping |
| `traceroute` | — | Rota izleme |
| `nslookup` | — | DNS sorgu |
| `hostname` | Tam | Hostname gösterme/ayarlama |
| `ifconfig` | — | Ağ arayüzü konfigürasyonu (legacy) |
| `ip` | — | Ağ arayüzü yönetimi (iproute2 subset) |
| `route` | — | Routing tablosu |
| `arp` | — | ARP tablosu |
| `telnet` | — | Telnet client (debug amaçlı) |

### 7.5 Process Utilities (10 applet)

| Applet | POSIX | Açıklama |
|--------|-------|----------|
| `ps` | Tam | Process listeleme |
| `kill` | Tam | Sinyal gönderme |
| `killall` | — | İsme göre sinyal gönderme |
| `top` | — | Process monitor (minimal) |
| `free` | — | Bellek kullanımı |
| `uptime` | — | Sistem çalışma süresi |
| `renice` | Tam | Öncelik değiştirme |
| `nice` | Tam | Öncelik ile çalıştırma |
| `nohup` | Tam | HUP engelle |
| `time` | Tam | Komut süresini ölç |

### 7.6 System Utilities (15 applet)

| Applet | POSIX | Açıklama |
|--------|-------|----------|
| `mount` | — | Filesystem bağlama |
| `umount` | — | Filesystem ayırma |
| `fdisk` | — | Disk bölümleme (minimal) |
| `mkfs.ext2` | — | ext2 filesystem oluşturma |
| `mkfs.vfat` | — | FAT filesystem oluşturma |
| `fsck` | — | Filesystem kontrolü |
| `swapon` | — | Swap aktifleştirme |
| `swapoff` | — | Swap devre dışı |
| `sysctl` | — | Kernel parametre yönetimi |
| `dmesg` | — | Kernel mesajları |
| `lsmod` | — | Yüklü kernel modülleri |
| `modprobe` | — | Kernel modül yükleme |
| `insmod` | — | Kernel modül ekleme |
| `rmmod` | — | Kernel modül kaldırma |
| `mdev` | — | Minimal device manager |

### 7.7 Editors & Diff (5 applet)

| Applet | POSIX | Açıklama |
|--------|-------|----------|
| `vi` | Tam | Minimal vi editörü |
| `diff` | Tam | Dosya farkları |
| `patch` | Tam | Patch uygulama |
| `ed` | Tam | Line editor |
| `cmp` | Tam | Byte karşılaştırma |

### 7.8 Miscellaneous (10 applet)

| Applet | POSIX | Açıklama |
|--------|-------|----------|
| `date` | Tam | Tarih/saat gösterme/ayarlama |
| `cal` | — | Takvim |
| `bc` | Tam | Hesap makinesi |
| `dc` | — | RPN hesap makinesi |
| `hexdump` | — | Hex görüntüleme |
| `od` | Tam | Octal dump |
| `seq` | — | Sayı dizisi |
| `which` | — | Komut yolu bulma |
| `logger` | — | Syslog'a yazdırma |
| `strings` | — | Binary'den string çıkarma |

### 7.9 Init & Service (5 applet)

| Applet | Açıklama |
|--------|----------|
| `init` | Minimal PID 1 init (container & embedded) |
| `halt` | Sistemi durdur |
| `reboot` | Yeniden başlat |
| `poweroff` | Kapat |
| `getty` | Terminal login |

**Toplam: ~130 applet**

---

## 8. Build System

### 8.1 Feature Flags

Her applet bağımsız bir cargo feature:

```toml
[features]
default = ["profile-standard"]

# Profiles
profile-minimal  = ["cat", "cp", "mv", "rm", "ls", "mkdir", "echo", "sh", "grep", "sed"]
profile-standard = ["profile-minimal", "awk", "find", "tar", "gzip", "wget", "vi", "top", "ps", ...]
profile-full     = ["profile-standard", "bc", "dc", "telnet", "fdisk", ...]
profile-container = ["profile-minimal", "wget", "tar", "gzip", "ps", "kill", "init"]
profile-embedded  = ["profile-minimal", "mount", "umount", "dmesg", "mdev", "init", "getty"]

# GNU extension toggle
gnu-extensions = []

# Shell extensions
shell-extended = []

# Individual applets
cat = []
ls = []
grep = []
# ... her applet için ayrı feature
```

### 8.2 Build Commands

```bash
# Varsayılan (standard profil)
cargo build --release

# Minimal gömülü profil
cargo build --release --features profile-embedded --no-default-features

# Container profil
cargo build --release --features profile-container --no-default-features

# Tek applet (debug)
cargo build --release --features "cat grep ls" --no-default-features

# Full + GNU extensions
cargo build --release --features "profile-full,gnu-extensions,shell-extended"

# Cross-compile (aarch64)
cargo build --release --target aarch64-unknown-linux-musl

# Cross-compile (MIPS, big endian)
cargo build --release --target mips-unknown-linux-musl
```

### 8.3 Post-Build Size Report

`cargo proteus-size` komutu (custom cargo subcommand):

```
╔══════════════════════════════════════════════════╗
║            Proteus Build Size Report             ║
╠══════════════════════════════════════════════════╣
║ Profile: standard                                ║
║ Target:  x86_64-unknown-linux-musl               ║
║ Total:   847 KB (stripped)                       ║
╠════════════════╦═════════╦═══════════════════════╣
║ Category       ║ Size    ║ Applets               ║
╠════════════════╬═════════╬═══════════════════════╣
║ proteus-core   ║  89 KB  ║ (shared)              ║
║ shell (nereus) ║ 142 KB  ║ sh                    ║
║ coreutils      ║ 156 KB  ║ 35 applets            ║
║ textutils      ║ 198 KB  ║ 20 applets            ║
║ fileutils      ║ 124 KB  ║ 18 applets            ║
║ netutils       ║  78 KB  ║ 12 applets            ║
║ procutils      ║  32 KB  ║ 10 applets            ║
║ sysutils       ║  18 KB  ║ 15 applets            ║
║ misc           ║  10 KB  ║ 10 applets            ║
╚════════════════╩═════════╩═══════════════════════╝

Top 10 largest applets:
  1. sh       142 KB   ████████████████████████
  2. awk       48 KB   ████████
  3. sed       32 KB   █████
  4. vi        28 KB   ████
  5. grep      24 KB   ████
  ...
```

### 8.4 Binary Optimization

- `lto = true` (Link Time Optimization)
- `opt-level = "z"` (size optimization)
- `strip = true`
- `panic = "abort"` (unwind tablosu yok)
- `codegen-units = 1` (daha iyi LTO)
- UPX sıkıştırma opsiyonel (gömülü için)

---

## 9. POSIX Compliance System

### 9.1 Compliance Levels

Her applet şu seviyelerden birinde:

| Level | Anlamı |
|-------|--------|
| `FULL` | POSIX.1-2017 tam uyumlu, tüm required flag'ler mevcut |
| `SUBSTANTIAL` | Tüm yaygın flag'ler mevcut, edge case'ler eksik olabilir |
| `PARTIAL` | Temel kullanım uyumlu, bazı flag'ler eksik |
| `NONE` | POSIX spesifikasyonu yok (GNU/Linux-specific araç) |

### 9.2 Runtime Compliance Query

```bash
$ proteus grep --posix-info
Applet:     grep
POSIX:      FULL (IEEE Std 1003.1-2017, XCU §grep)
GNU ext:    enabled (--color, -r, -P)
Flags:
  POSIX:    -E -F -c -e -f -i -l -n -q -s -v -x  [ALL SUPPORTED]
  GNU:      -r -R --color --include --exclude -P   [ENABLED]
  Missing:  (none)
```

### 9.3 Compliance Test Suite

```bash
$ cargo test --features posix-tests

Running POSIX compliance tests...
  grep: 147/147 passed ✓
  sed:  203/203 passed ✓
  awk:  312/312 passed ✓
  sh:   589/589 passed ✓
  ...
Total: 4,821/4,821 POSIX tests passed
```

---

## 10. Security Architecture

### 10.1 Seccomp Profiles

Compile-time tanımlı, her applet için ayrı BPF filtre:

```rust
// applets/coreutils/cat.rs
#[applet(
    name = "cat",
    posix = "full",
    seccomp = ["read", "write", "open", "openat", "close", "fstat", "mmap", "munmap", "exit_group"]
)]
pub fn main(args: Args) -> i32 {
    // ...
}
```

### 10.2 Capability Model

```
proteus init    → CAP_SYS_ADMIN, CAP_NET_ADMIN, ...
proteus mount   → CAP_SYS_ADMIN
proteus ping    → CAP_NET_RAW
proteus cat     → (hiçbiri)
proteus wget    → CAP_NET_BIND_SERVICE (opsiyonel)
```

### 10.3 Sandbox Modes

```bash
# Strict mode (varsayılan): seccomp + capability drop
proteus --sandbox=strict cat /etc/passwd

# Permissive mode: sadece capability drop
proteus --sandbox=permissive cat /etc/passwd

# Off: hiçbir kısıtlama yok
proteus --no-sandbox cat /etc/passwd
```

---

## 11. Testing Strategy

### 11.1 Test Layers

1. **Unit tests:** Her fonksiyon için
2. **Applet tests:** Her applet'in kendi integration test suite'i
3. **POSIX conformance tests:** POSIX.1-2017 spesifikasyonundan türetilmiş test vektörleri
4. **BusyBox compatibility tests:** BusyBox test suite'inden uyarlanmış
5. **Fuzz testing:** `cargo-fuzz` ile her text processing applet'i (grep, sed, awk, sh)
6. **Cross-architecture tests:** QEMU usermode üzerinde ARM, MIPS, RISC-V

### 11.2 CI Pipeline

```
┌─────────────────────────────────────────────────────────┐
│                    CI Pipeline                          │
├─────────────────────────────────────────────────────────┤
│  1. cargo fmt --check                                   │
│  2. cargo clippy -- -D warnings                         │
│  3. cargo test (host architecture)                      │
│  4. cargo test --features posix-tests                   │
│  5. Cross-compile: x86_64, aarch64, armv7, mips, riscv  │
│  6. QEMU test (aarch64, armv7)                          │
│  7. Fuzz (nightly, grep/sed/awk/sh)                     │
│  8. Size report + regression check                      │
│  9. cargo audit (vulnerability scan)                    │
│ 10. POSIX compliance report generation                  │
└─────────────────────────────────────────────────────────┘
```

### 11.3 Size Regression

CI her build'de binary boyutunu ölçer. Eğer bir PR binary boyutunu %5'ten fazla artırıyorsa otomatik uyarı verilir. Applet bazında boyut artışı raporlanır.

---

## 12. Cross-Platform Support

### 12.1 Tier 1 (Tam destek, CI'da test)

| Target | Arch | Libc |
|--------|------|------|
| `x86_64-unknown-linux-musl` | x86_64 | musl (static) |
| `aarch64-unknown-linux-musl` | ARM64 | musl (static) |
| `armv7-unknown-linux-musleabihf` | ARMv7 | musl (static) |

### 12.2 Tier 2 (Derlenir, temel test)

| Target | Arch | Libc |
|--------|------|------|
| `mips-unknown-linux-musl` | MIPS (BE) | musl |
| `mipsel-unknown-linux-musl` | MIPS (LE) | musl |
| `riscv64gc-unknown-linux-musl` | RISC-V 64 | musl |
| `x86_64-unknown-freebsd` | x86_64 | FreeBSD libc |

### 12.3 Tier 3 (Best effort)

| Target | Arch |
|--------|------|
| `aarch64-apple-darwin` | macOS ARM64 |
| `x86_64-apple-darwin` | macOS x86_64 |
| `x86_64-unknown-netbsd` | NetBSD |
| `x86_64-unknown-openbsd` | OpenBSD |

### 12.4 Platform Abstraction

```rust
// proteus-core/src/platform/mod.rs
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "freebsd")]
mod freebsd;
#[cfg(target_os = "macos")]
mod macos;

pub trait Platform {
    fn mount(source: &str, target: &str, fstype: &str, flags: u64) -> Result<()>;
    fn sysctl(name: &str) -> Result<String>;
    fn get_process_list() -> Result<Vec<ProcessInfo>>;
    // ...
}
```

---

## 13. Documentation

### 13.1 Man Pages

Her applet için POSIX-format man page (roff):

```bash
proteus man ls       # Yerleşik man page görüntüleyici
proteus ls --help    # Kısa yardım
proteus ls --help-full  # Detaylı yardım (man page'e eşdeğer)
```

### 13.2 Yerleşik Dokümantasyon

Man page'ler binary'ye gömülü (compile-time include). `--help` kısa özet, `--help-full` tam dokümantasyon verir. `feature = "embedded-docs"` ile aktif, devre dışı bırakılarak ~100KB tasarruf edilebilir.

### 13.3 Web Dokümantasyon

- Applet referansı (her applet için flag listesi, örnekler)
- POSIX compliance matrisi
- BusyBox migration guide
- Cross-compilation rehberi
- Contributing guide

---

## 14. Performance Targets

| Metrik | Hedef | BusyBox Referans |
|--------|-------|------------------|
| `cat` (1GB dosya) | ≤ BusyBox süresi | ~2.1s |
| `grep` (100MB, basit pattern) | ≤ BusyBox süresi | ~0.8s |
| `sort` (10M satır) | ≤ 1.5x BusyBox | ~12s |
| `sh` startup | ≤ 5ms | ~3ms |
| `ls -la /usr` | ≤ BusyBox süresi | ~15ms |
| Binary boyut (minimal) | ≤ 500 KB | ~400 KB |
| Binary boyut (standard) | ≤ 1 MB | ~900 KB |
| Binary boyut (full) | ≤ 2 MB | ~1.5 MB |
| RAM (sh idle) | ≤ 1 MB RSS | ~600 KB |

---

## 15. Versioning & Release Strategy

### 15.1 Versioning

Semantic versioning: `MAJOR.MINOR.PATCH`

- **MAJOR:** Breaking API/ABI değişikliği, applet davranış değişikliği
- **MINOR:** Yeni applet, yeni flag, performance improvement
- **PATCH:** Bug fix, güvenlik yaması

### 15.2 Release Milestones

| Milestone | İçerik | Hedef |
|-----------|--------|-------|
| v0.1 (Alpha) | Core library + 10 coreutils (cat, ls, cp, mv, rm, mkdir, echo, head, tail, wc) + sh (basic) | M1 |
| v0.2 (Alpha) | Text processing (grep, sed, sort, cut, tr, uniq) + argument parser finalize | M2 |
| v0.3 (Alpha) | File utilities (find, xargs, tar, gzip) + awk | M3 |
| v0.4 (Beta) | Network (wget, ping, nc) + process (ps, kill, top) + seccomp | M4 |
| v0.5 (Beta) | System utils (mount, umount, dmesg) + init + mdev | M5 |
| v0.6 (Beta) | Shell (Nereus) tam POSIX uyumluluk + line editing | M6 |
| v0.7 (RC) | vi editor + diff/patch + misc applets | M7 |
| v0.8 (RC) | POSIX conformance test suite tamamlama | M8 |
| v0.9 (RC) | Cross-platform (FreeBSD, macOS) + performance tuning | M9 |
| v1.0 (Stable) | Full test coverage, docs, size optimization, release | M10 |

### 15.3 Release Artifacts

Her release için:
- Statik binary'ler: x86_64, aarch64, armv7, mips, riscv64
- Docker imajı: `proteus:latest` (scratch tabanlı, sadece proteus binary)
- OCI imajı: Container runtime'lar için
- Source tarball
- SBOM (Software Bill of Materials)
- POSIX compliance raporu
- Size raporu

---

## 16. Competitive Landscape

| Özellik | BusyBox | Toybox | uutils | **Proteus** |
|---------|---------|--------|--------|-------------|
| Dil | C | C | Rust | **Rust** |
| Memory safety | Hayır | Hayır | Evet | **Evet** |
| POSIX compliance | Kısmi | Kısmi | Kısmi | **Tam (hedef)** |
| GNU extensions | Kısmi | Hayır | Evet | **Opsiyonel** |
| Tek binary | Evet | Evet | Hayır | **Evet** |
| Gömülü odaklı | Evet | Evet | Hayır | **Evet** |
| Unicode | Kısmi | Kısmi | Evet | **Evet** |
| Seccomp | Hayır | Hayır | Hayır | **Evet** |
| Shell dahil | Evet (ash) | Evet (toysh) | Hayır | **Evet (Nereus)** |
| Boyut raporu | Hayır | Hayır | N/A | **Evet** |
| `no_std` | N/A | N/A | Hayır | **Evet (core)** |

---

## 17. Dependencies Policy

### 17.1 İzin Verilen Bağımlılıklar

Sıfır veya minimum dış bağımlılık hedeflenir. İzin verilenler:

| Crate | Amaç | Koşul |
|-------|-------|-------|
| `rustls` | TLS (wget) | `feature = "tls"` |
| `miniz_oxide` | Deflate/gzip | Saf Rust, no_std uyumlu |
| `libc` | Syscall binding | Sadece platform abstraction |

### 17.2 Yasaklı Bağımlılıklar

- `clap`, `structopt`: Çok büyük, kendi parser kullanılır
- `tokio`, `async-std`: Async runtime yok, blocking I/O
- `serde`, `serde_json`: Gereksiz, elle parse
- `regex` (crate): Kendi regex engine kullanılır
- `openssl`: rustls tercih edilir

---

## 18. Contributing & Community

### 18.1 Contribution Model

- Her yeni applet bir PR
- PR template: POSIX flag listesi, test coverage, size impact
- `cargo proteus-size` çıktısı PR'a eklenmeli
- Unsafe kod PR'ları ayrı review süreci

### 18.2 Applet Ekleme Rehberi

Yeni applet eklemek için:

1. `applets/<category>/<name>.rs` oluştur
2. `#[applet(...)]` macro ile metadata tanımla
3. `Cargo.toml`'a feature flag ekle
4. POSIX test vektörlerini yaz
5. Man page yaz
6. `cargo proteus-size` ile boyut etkisini ölç

---

## 19. Proje Yapısı

```
proteus/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── LICENSE-MIT
├── LICENSE-APACHE
├── CHANGELOG.md
├── CONTRIBUTING.md
├── POSIX-COMPLIANCE.md
│
├── src/
│   └── main.rs                  # Entry point + dispatch
│
├── core/                        # proteus-core crate
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── args.rs
│       ├── io.rs
│       ├── utf8.rs
│       ├── locale.rs
│       ├── regex.rs
│       ├── glob.rs
│       ├── error.rs
│       ├── sandbox.rs
│       └── platform/
│           ├── mod.rs
│           ├── linux.rs
│           ├── freebsd.rs
│           └── macos.rs
│
├── applets/                     # proteus-applets crate
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── coreutils/
│       │   ├── mod.rs
│       │   ├── cat.rs
│       │   ├── ls.rs
│       │   ├── cp.rs
│       │   └── ...
│       ├── textutils/
│       ├── fileutils/
│       ├── netutils/
│       ├── procutils/
│       ├── sysutils/
│       ├── editors/
│       └── misc/
│
├── shell/                       # proteus-shell (Nereus) crate
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── lexer.rs
│       ├── parser.rs
│       ├── interpreter.rs
│       ├── builtins.rs
│       ├── line_editing.rs
│       └── completion.rs
│
├── build-tools/                 # Build utilities
│   ├── size-report/
│   └── compliance-check/
│
├── tests/
│   ├── posix/                   # POSIX conformance tests
│   ├── compat/                  # BusyBox compatibility tests
│   ├── integration/             # End-to-end tests
│   └── fuzz/                    # Fuzz targets
│
├── docs/
│   ├── man/                     # Man pages (roff)
│   ├── migration/               # BusyBox migration guide
│   └── architecture/            # Architecture decisions
│
├── docker/
│   └── Dockerfile               # scratch-based container
│
└── .github/
    └── workflows/
        ├── ci.yml
        ├── cross-compile.yml
        ├── fuzz.yml
        └── release.yml
```

---

## Appendix A: ASCII Art Logo

```
    ____             __
   / __ \_________  / /____  __  _______
  / /_/ / ___/ __ \/ __/ _ \/ / / / ___/
 / ____/ /  / /_/ / /_/  __/ /_/ (__  )
/_/   /_/   \____/\__/\___/\__,_/____/

  Shape-shifting Unix toolkit
  One binary. Every tool. Memory safe.
```

---

## Appendix B: Quick Reference

```bash
# Install
curl -L https://github.com/user/proteus/releases/latest/download/proteus-$(uname -m) -o /usr/local/bin/proteus
chmod +x /usr/local/bin/proteus
proteus --install -s /usr/local/bin

# Usage
proteus ls -la
proteus --list
proteus grep --posix-info

# Build from source
git clone https://github.com/user/proteus.git
cd proteus
cargo build --release --features profile-standard
```
