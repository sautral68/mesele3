// mesele3 - Sezar şifri çözüji (RTF faýl üçin, UNICODE)
//
// Algoritm:
//   1. Sezar.rtf okamak (\uXXXX? escape-lerini çykar)
//   2. k = 1..19999 üçin şifri çöz, türkmen dili scoringini hasapla
//   3. Iň ýokary score - açar k
//   4. k çap et, dikeldilen.rtf döret
//
// Kompilasiýa: cargo build --release
// Iş: ./mesele3   (Sezar.rtf şol katalogda bolmaly)

use std::fs;
use std::io::{self, Write};

// ─── RTF-den Unicode kod nokatlary ───────────────────────────────────────────

fn extract_codepoints(rtf_bytes: &[u8]) -> Vec<u32> {
    let text = String::from_utf8_lossy(rtf_bytes);
    let mut result = Vec::with_capacity(text.len());
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch != '\\' {
            // RTF topar belgileri - geç
            if ch != '{' && ch != '}' && !ch.is_ascii_control() {
                result.push(ch as u32);
            }
            continue;
        }

        // Backslash tapyldy
        match chars.peek().copied() {
            Some('u') => {
                chars.next(); // 'u'
                let mut num = String::new();
                if chars.peek() == Some(&'-') {
                    num.push('-');
                    chars.next();
                }
                while chars.peek().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                    num.push(chars.next().unwrap());
                }
                // Placeholder ? ýa-da harp geç
                if chars.peek() == Some(&'?') {
                    chars.next();
                }
                if let Ok(n) = num.parse::<i32>() {
                    let cp = if n < 0 { (n + 65536) as u32 } else { n as u32 };
                    result.push(cp);
                }
            }
            Some('\\') | Some('{') | Some('}') => {
                result.push(chars.next().unwrap() as u32);
            }
            Some('\n') | Some('\r') => {
                chars.next();
                result.push(b'\n' as u32);
            }
            _ => {
                // RTF gözegçilik sözi: bölüjä çenli geç
                while chars
                    .peek()
                    .map(|c| !matches!(c, ' ' | '{' | '}' | '\\' | '\n' | '\r'))
                    .unwrap_or(false)
                {
                    chars.next();
                }
                // Bir boşluk bölüji-geçirijisi
                if chars.peek() == Some(&' ') {
                    chars.next();
                }
            }
        }
    }

    result
}

// ─── Sezar deşifrleme ────────────────────────────────────────────────────────

fn decrypt(codepoints: &[u32], k: u32) -> Vec<u32> {
    codepoints
        .iter()
        .map(|&cp| if cp >= k { cp - k } else { cp })
        .collect()
}

// ─── Türkmen dili scoring ─────────────────────────────────────────────────────

fn score(cps: &[u32]) -> f64 {
    if cps.len() < 100 {
        return 0.0;
    }
    let n = cps.len() as f64;
    let mut s = 0.0f64;

    // 1. Boşluk gatnaşygy (10-20% gowy)
    let spaces = cps.iter().filter(|&&c| c == 32).count() as f64;
    let sp = spaces / n;
    s += match () {
        _ if sp > 0.10 && sp < 0.20 => 40.0,
        _ if sp > 0.07 && sp < 0.25 => 20.0,
        _ if sp > 0.04 && sp < 0.30 => 5.0,
        _ => -20.0,
    };

    // 2. ASCII harplar (a-z, A-Z) gatnaşygy
    let ascii_letters = cps
        .iter()
        .filter(|&&c| (c >= 65 && c <= 90) || (c >= 97 && c <= 122))
        .count() as f64;
    let al = ascii_letters / n;
    s += match () {
        _ if al > 0.55 && al < 0.80 => 40.0,
        _ if al > 0.45 && al < 0.85 => 20.0,
        _ if al > 0.35 => 5.0,
        _ => -10.0,
    };

    // 3. Türkmen ýörite harplary (Ä Ç Ň Ö Ş Ü Ý Ž we kiçileri)
    let turkmen_specials: &[u32] = &[
        0xC4, 0xE4, // Ä ä
        0xC7, 0xE7, // Ç ç
        0x147, 0x148, // Ň ň
        0xD6, 0xF6, // Ö ö
        0x15E, 0x15F, // Ş ş
        0xDC, 0xFC, // Ü ü
        0xDD, 0xFD, // Ý ý
        0x17D, 0x17E, // Ž ž
    ];
    let spec = cps
        .iter()
        .filter(|&&c| turkmen_specials.contains(&c))
        .count() as f64;
    let sp2 = spec / n;
    s += match () {
        _ if sp2 > 0.01 && sp2 < 0.12 => 50.0, // ← iň möhüm görkeziji
        _ if sp2 > 0.005 && sp2 < 0.18 => 25.0,
        _ if sp2 > 0.0 => 5.0,
        _ => 0.0,
    };

    // 4. Çap edilip bilinýän simwollar
    let printable = cps.iter().filter(|&&c| c >= 32 && c < 0x10000).count() as f64;
    if printable / n > 0.97 {
        s += 15.0;
    } else if printable / n < 0.85 {
        s -= 30.0;
    }

    // 5. Gaty uly kod nokatlary jezalandyr
    let high = cps.iter().filter(|&&c| c > 0x5000).count() as f64;
    s -= (high / n) * 100.0;

    // 6. Gözegçilik simwollary jezalandyr
    let ctrl = cps
        .iter()
        .filter(|&&c| c < 32 && c != 9 && c != 10 && c != 13)
        .count() as f64;
    s -= (ctrl / n) * 80.0;

    // 7. Türkmen ýygylyk harplary (a, e, i, n, r, l, y, ý, ä)
    let freq: &[u32] = &[
        b'a' as u32, b'e' as u32, b'i' as u32, b'n' as u32,
        b'r' as u32, b'l' as u32, b'y' as u32, b'd' as u32,
        b'm' as u32, b'k' as u32,
        0xFD, 0xDD, // ý Ý
        0xE4, 0xC4, // ä Ä
        0x148, 0x147, // ň Ň
    ];
    let freq_count = cps.iter().filter(|&&c| freq.contains(&c)).count() as f64;
    let fq = freq_count / n;
    if fq > 0.20 {
        s += 20.0;
    }

    s
}

// ─── RTF ýaz ─────────────────────────────────────────────────────────────────

fn write_rtf(path: &str, cps: &[u32]) -> std::io::Result<()> {
    let mut out = String::with_capacity(cps.len() * 3);

    // RTF başy
    out.push_str("{\\rtf1\\ansi\\ansicpg65001\\deff0\n");
    out.push_str("{\\fonttbl{\\f0\\fswiss\\fcharset204 Arial;}}\n");
    out.push_str("\\f0\\fs24\\lang1074 ");

    for &cp in cps {
        match cp {
            10 => out.push_str("\\par\n"),
            13 => {} // geç
            92 => out.push_str("\\\\"), // backslash
            123 => out.push_str("\\{"),
            125 => out.push_str("\\}"),
            0..=127 => {
                if let Some(c) = char::from_u32(cp) {
                    out.push(c);
                }
            }
            _ => {
                // Unicode escape
                let n: i32 = if cp > 32767 {
                    cp as i32 - 65536
                } else {
                    cp as i32
                };
                out.push_str(&format!("\\u{}?", n));
            }
        }
    }

    out.push('}');
    fs::write(path, out.as_bytes())
}

// ─── MAIN ─────────────────────────────────────────────────────────────────────

fn main() {
    // Faýl ýoluny argument ýa-da interaktiw sora
    let args: Vec<String> = std::env::args().collect();
    let rtf_path = if args.len() > 1 {
        args[1].clone()
    } else {
        print!("Sezar.rtf faýlynyň ýoluny giriziň: ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let trimmed = input.trim().to_string();
        if trimmed.is_empty() {
            "Sezar.rtf".to_string()
        } else {
            trimmed
        }
    };

    let start = std::time::Instant::now();
    println!("'{}' okalýar...", rtf_path);
    let data = match fs::read(&rtf_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("ÝALŇYŞLYK: {} okap bolmady: {}", rtf_path, e);
            std::process::exit(1);
        }
    };
    println!("Faýl okaldy ({} baýt).", data.len());

    println!("RTF-den Unicode simwollar çykarylýar...");
    let cps = extract_codepoints(&data);
    println!("Jemi {} simwol çykaryldy.", cps.len());

    if cps.is_empty() {
        eprintln!("ÝALŇYŞLYK: Simwol çykaryp bolmady. RTF formatyny barlaň.");
        std::process::exit(1);
    }

    // Tizlik üçin ilkinji N simwola bak
    let sample_size = cps.len().min(3000);
    let sample = &cps[..sample_size];

    println!(
        "\nk = 1..19999 üçin gözleg başlanýar ({} simwol nusgasy bilen)...",
        sample_size
    );

    let mut best_k = 1u32;
    let mut best_score = f64::NEG_INFINITY;
    let mut top10: Vec<(f64, u32)> = Vec::new();

    for k in 1u32..20000 {
        let dec = decrypt(sample, k);
        let sc = score(&dec);

        if sc > best_score {
            best_score = sc;
            best_k = k;
        }

        // Iň gowy 10-y sakla
        if top10.len() < 10 || sc > top10.last().map(|x| x.0).unwrap_or(f64::NEG_INFINITY) {
            top10.push((sc, k));
            top10.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
            top10.truncate(10);
        }

        if k % 2000 == 0 {
            print!(
                "\r  {}/19999 barlandy... Iň gowy: k={} (score={:.1})   ",
                k, best_k, best_score
            );
            io::stdout().flush().unwrap();
        }
    }
    println!();

    // Netijeler
    println!("\n=== IŇ GOWY 10 AÇAR ===");
    println!("{:<8} {:<10}", "k", "Score");
    println!("{}", "-".repeat(20));
    for (sc, k) in &top10 {
        println!("{:<8} {:<10.2}", k, sc);
    }

    println!("\n╔══════════════════════╗");
    println!("║  AÇAR:  k = {:6}   ║", best_k);
    println!("╚══════════════════════╝");

    // Doly teksti çöz
    println!("\nDoly tekst k={} bilen dikeldilýär...", best_k);
    let full = decrypt(&cps, best_k);

    // Ilkinji 500 simwol
    let preview: String = full
        .iter()
        .take(500)
        .filter_map(|&c| char::from_u32(c))
        .collect();
    println!("\n--- Ilkinji 500 simwol ---");
    println!("{}", preview);
    println!("---");

    // dikeldilen.rtf ýaz
    println!("\ndikeldilen.rtf ýazylýar...");
    match write_rtf("dikeldilen.rtf", &full) {
        Ok(()) => println!("OK: dikeldilen.rtf saklandi."),
        Err(e) => eprintln!("ÝALŇYŞLYK: dikeldilen.rtf ýazyp bolmady: {}", e),
    }

    println!("\n=== TAÝÝAR ===");
    println!("  Açar:           k = {}", best_k);
    println!("  Dikeldilen faýl: dikeldilen.rtf");
    println!("  Geçen wagt:      {:.2} sekunt", start.elapsed().as_secs_f64());
}