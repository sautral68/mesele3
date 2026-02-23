# mesele3 - Sezar şifri çözüji

## 1. Rust-y nädip gurmaly (установка)

### Windows
1. Şu salga git: **https://rustup.rs**
2. `rustup-init.exe` faýlyny ýükle we işlet
3. Gurnama tamamlananda terminal aç we barla:
   ```
   rustc --version
   cargo --version
   ```

### Linux / macOS
Terminala şuny ýaz:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
Soňra terminaly täzeden aç.

---

## 2. Programma nädip ýygnamaly (компиляция)

`mesele3` papkasyna git:
```bash
cd mesele3
cargo build --release
```

Taýýar faýl şu ýerde bolar:
- **Windows:** `target\release\mesele3.exe`
- **Linux/Mac:** `target/release/mesele3`

---

## 3. Nädip işletmeli (запуск)

`Sezar.rtf` faýlyny `mesele3.exe` bilen bir papka goý, soňra:

```bash
# Windows:
mesele3.exe

# Linux/Mac:
./target/release/mesele3

# Faýlyň adyny görkezip:
mesele3.exe C:\Users\sen\Sezar.rtf
```

---

## 4. Programma näme çykarýar

```
'Sezar.rtf' okalýar...
Faýl okaldy (XXXXXX baýt).
RTF-den Unicode simwollar çykarylýar...
Jemi XXXX simwol çykaryldy.

k = 1..19999 üçin gözleg başlanýar...
  19999/19999 barlandy... Iň gowy: k=XXXXX (score=XXX.X)

=== IŇ GOWY 10 AÇAR ===
k        Score
--------------------
XXXXX    XXX.XX
...

╔══════════════════════╗
║  AÇAR:  k = XXXXX   ║
╚══════════════════════╝

--- Ilkinji 500 simwol ---
(dikeldilen tekstiň başy görkezilýär)
---

dikeldilen.rtf ýazylýar...
OK: dikeldilen.rtf saklandy.

=== TAÝÝAR ===
  Açar:            k = XXXXX
  Dikeldilen faýl: dikeldilen.rtf
  Geçen wagt:      1.84 sekunt
```

---

## 5. Netije faýllar

| Faýl | Mazmuny |
|------|---------|
| `dikeldilen.rtf` | Açylan (dekodlanan) türkmen teksti |

---

## 6. Algoritm nähili işleýär

1. `Sezar.rtf` içindäki `\uXXXX?` formatly Unicode simwollary çykarýar
2. k = 1 … 19 999 üçin her k-ny synap görýär
3. Her dekodlanan tekste **türkmen dili scoring** berýär:
   - Boşluk gatnaşygy 10–20% → +40 bal
   - ASCII harplar 55–80% → +40 bal
   - **Türkmen ýörite harplary** (Ä Ç Ň Ö Ş Ü Ý Ž) → **+50 bal** ← esasy görkeziji
   - Uly Unicode kodlar we gözegçilik simwollary → minus bal
4. Iň ýokary score alan k = açar
5. Doly tekst şol k bilen dekodlanýar → `dikeldilen.rtf`

**Tizlik:** ~1–3 sekunt (ilki 3000 simwol bilen barlaýar, soňra doly teksti çözýär)

---

## 7. Dependency (baglylyklarlar)

**Hiç zat gerek däl!** Diňe Rust özüniň standart kitaphanasy ulanylýar.
`Cargo.toml`-da goşmaça dependency ýok.