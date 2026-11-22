# velvet73 — CIRCUIT.md (registro técnico da cadeia modelada)

> Rascunho v0 — 2026-11-17. Sem código de plugin. Cada parâmetro tem fonte e nível:
> **[P]** primária (manual, patente, paper com medição, esquema) · **[S]** secundária séria (técnicos, fóruns de restauração) ·
> **[C]** calculado por nós em `analysis/` e validado contra [P] · **[H]** heurística (instrumentos parecidos, física), a confirmar.
> Alvo: **Rhodes Mark I Stage 73, ~1973–75 → Fender Twin Reverb AA270**.

```
tecla → martelo+neoprene → tine (modal) ⇄ tone bar (modal) → ponta do tine em Φ(y,gap) → v = dΦ/dt
      → 73 fontes somadas na harpa (série/paralelo, R+L) → nameboard (Bass Boost + Volume) → cabo (C)
      → Twin: 7025 → TMB → mix → reverb/vibrato → PI 12AT7 → 4×6L6GC → OT → 2×12 → mic
```

---

## 1. Martelo e ação

| Parâmetro | Valor | Nível |
|---|---|---|
| Lei de contato | F = K·c^β + histerese α·d(c^β)/dt (Hunt-Crossley/Stulov) | [P] Falaize & Hélie 2017 |
| Expoente β | 2,5 | [P] Falaize (valor "típico" de piano, não medido no Rhodes) → [H] |
| Massa efetiva do martelo | 30 g (Falaize) | [H], a calibrar pela duração do contato |
| Dureza por zona (88): 1–30 / 31–40 / 41–50 / 51–64 / 65–88 | 30 / 50 / 70 / 90 Shore A / "wrapped" | [P] manual cap. 4 |
| Altura da ponta por zona | 6,35 / 7,94 / 9,5 / 11,1 / 11,1 mm | [P] |
| Escapement | grave 6,35–9,5 · nota 41 1,6–3,2 · agudo 0,8–2,4 mm | [P] |
| Striking line | 57 mm (grave) → 3 mm (agudo) da borda do tone generator | [P] |
| Golpe ajustado para excitar "substancialmente só a fundamental" | — | [P] US 4,040,321 |
| Double stroke (grave, escapement curto); martelo abafando (escapement 0) | fenômenos a reproduzir | [P] manual cap. 10 |
| Força de golpe medida (martelo instrumentado) | 1,4–6,7 N | [P] Gabrielli (faixa de referência) |

## 2. Tine

| Parâmetro | Valor | Nível |
|---|---|---|
| Comprimento por tecla | `docs/data/tine_lengths.csv` (carta de corte do manual, escala 0,254 mm/px) | [C] validado: F1 154 vs 150 mm, F3 78 vs 76 mm (Gabrielli) [P] |
| Diâmetro | 1,5 mm (Shear, Lover 0,060"); Gabrielli cota ~1 mm na ponta → conicidade | [P] / [?] perfil |
| Material | aço carbono forjado (swaged), E ≈ 200 GPa, ρ ≈ 7850 | [P] patentes / manual |
| Teclas 1–7 (só 88) | sem tone bar, mesmo comprimento da 8; razões de modo diferentes (9,6 / 29 / 52) | [P] Gabrielli §VI-A |
| Mola de afinação | ±1,5 tom; 1 cm de deslocamento = ×1,09 em f0 (F3); altera as razões dos modos altos (m3: 20 → 17) | [P] manual; [P] Gabrielli Fig. 15 |
| Amplitude estacionária da ponta | ~3,2 mm em C3 | [P] Lover; [C] confirmado no ajuste a Shear (3,0 mm) |
| 2ª polarização (horizontal) | menor que a vertical; vem de imperfeição do golpe | [P] Pfeifle (câmera) |

## 3. Modos do diapasão (`docs/data/modes.csv`)

| Modo | Razão média ± desvio (teclado todo, Mk I 73) | Origem física | Nível |
|---|---|---|---|
| sub | F1: 0,83; A2 e F3: 0,58 (v0.4: interpolado em log f, ±0,04 por garfo; antes sorteado em 0,48 ± 0,19) | **tone bar** | [P] Gabrielli |
| 1 | 1 | tine + tone bar | — |
| 2 | 7,1 ± 0,3 | tine (viga ideal seria 6,27) | [P] |
| 3 | 20,4 ± 0,4 | tine (ideal 17,55) | [P] |
| 4 | 39,7 ± 0,9 | tine (ideal 34,4) | [P] |
| 5, 6 | 62,9 / 93,1 | só nas teclas mais graves | [P] |
| Teto | nenhum modo acima de ~10 kHz | | [P] |

Decaimentos:
- **Fundamental:** dos Q de Shear, que dão 2,2 / 5,8 / 5,6 / 7,8 / 19,3 dB/s em 78 / 156 / 311 / 622 / 1244 Hz [P]. Gabrielli mede 3–7,5 dB/s no conjunto montado e 5–83 dB/s **sem tone bar** [P]. A diferença entre pianos (fator ~2–3) vira parâmetro de condição [H].
- **Modos inarmônicos:** rápidos, 21–294 dB/s (Gabrielli, tabela II); ajuste fraco de ~3,8·f^0,41 [H]. A tone bar **encurta** os inarmônicos e **alonga** a fundamental [P]. Isso é o "decaimento primeiro rápido, depois gradual" da patente de 1961 [P].
- **Sidebands:** **não estão no movimento do tine** (laser). Surgem da intermodulação no captador, espaçadas de f0 em torno dos modos 2/3/4, e são perceptualmente relevantes [P]. O nosso modelo gera isso de graça, por ser não-linear.
- **Harpa de madeira:** modos de 10–100 Hz, que aparecem só na tone bar (vertical), não no tine [P]. Para o sinal elétrico são desprezíveis.

## 4. Captador

| Parâmetro | Valor | Nível |
|---|---|---|
| Geometria do polo | tronco de cone: face plana r = 0,039", base r = 0,145", altura 0,127" | [P] desenho US 4,040,321 |
| Ímã | Alnico 5, 0,5" × 0,1875", propositalmente fraco | [P] |
| Campo | cargas superficiais; reproduz as curvas medidas da patente com o sensor recuado 5 mm | [C] |
| Fluxo | Φ ∝ Bz na ponta do tine (melhor ajuste); ∫Bz² ao longo do tine quase igual | [C] |
| Validação 1 | espectros de Shear (G3, no eixo/5 mm): erro ≈ 3 dB/harmônico com gap 3,2 mm, A 3 mm | [C] |
| Validação 2 | modelo Pfeifle-Münster usado por Gabrielli (áudio "fund-only"): F3 erro 2,1 dB; A2 1,5 dB | [C] |
| Timbre (offset) | fábrica "levemente acima do centro" → ~1–1,5 mm | [P] manual + [C] |
| Gap | 1,6–3,2 mm; 0,5 mm no médio/agudo a partir de 03/1972 | [P] |
| DC por captador | 170–190 Ω | [P] fenderrhodes.com / Vintage Vibe |
| Fio / espiras | AWG 37 (0,0045"), < 2500 espiras | [S] EP-Forum |

## 5. Harpa (rede elétrica)

| Parâmetro | Valor | Nível |
|---|---|---|
| Fiação | 1 grupo de 4 + 23 grupos de 3 em paralelo, em série; polaridade alternada por grupo (reduz hum) | [P] manual fig. 10-4; [P] Shear |
| R DC total | ≈ 45 + 23×60 = **1425 Ω** | [C] a partir de [P] |
| Impedância de saída | ≈ 2500 Ω (manual, frequência não dita; Gabrielli repete) | [P] |
| **Indutância total** | **nominal 0,5 H, faixa 0,33–0,85 H** | [H] veja abaixo |

A indutância sai de três caminhos independentes:
- Se os 2500 Ω forem a 1 kHz, |Z| contra 1425 Ω DC dá **0,33 H**.
- Uma medição de fórum numa harpa Stage 88 de 1973 deu 854 mH com 2,1 kΩ. Escalando pelo número de grupos em série (1425/2100), isso dá **~0,58 H** para a de 73.
- Uma bobina de AWG 37 com ~2500 espiras sobre um núcleo de Alnico/aço de ~5 mm dá 40–150 mH por captador. Com o fator da harpa (L_total ≈ 7,9·L), isso dá 0,3–1,2 H.
- **Controle sugerido:** "Harp" com faixa de 0,33–0,85 H, ou fixo em 0,5 H.

Capacitância própria dos captadores e da fiação da harpa: desprezível perto do cabo [H].

## 6. Nameboard do Stage (passivo)

Topologia: harpa → **reostato 50k reverse-audio ("Bass Boost") em paralelo com 47 nF** → **pot 10k audio ("Volume")** ao terra, saída no wiper.
- Fonte: esquema redesenhado de pianos de 1979–84 no EP-Forum; Vintage Vibe confirma os valores 10k / 50k / 0,047 µF [S].
- Pianos **anteriores (~1969–72, sem placa frontal)** usavam 10k + 10k + 1 µF [S]. Sobre o que isso faz há divergência entre técnicos [?].
- Para 1973–75 assumo a versão 50k/47nF [H, provável].

Resultados calculados (`analysis/stage_network.py`, L = 0,5 H, cabo de 500 pF, entrada de 1 MΩ):

| Situação | Resultado | Confere com |
|---|---|---|
| Bass Boost no máximo | plano | Gabrielli: "sem atenuação no máximo" [P] |
| Bass Boost no mínimo | shelf de −12,8 dB em 50 Hz, −3,5 dB em 300 Hz, assíntota DC −14,6 dB; polo ≈ 340 Hz | Gabrielli: "low shelving 18 dB bass cut, cutoff 300 Hz" [P]; os 3 dB de diferença na profundidade ficam em aberto |
| **Volume 10k carregando a harpa indutiva** | **passa-baixa −3 dB em ~4,1 kHz** (2,4 kHz com 0,85 H; 6,4 kHz com 0,33 H) | Engenheiro da Rhodes: "o pot de 10k filtra o topo; 25k seria melhor" [S] |
| Volume reduzido | quase não muda o corte (3,7–4,1 kHz) | |
| Harpa direto em 1 MΩ, sem nameboard | **pico ressonante em ~10 kHz, +22 dB** (sem amortecimento) | O som de "DI direto da harpa" fica bem mais brilhante [S] |
| Cabo de 150 → 1500 pF | corte de 3,8 → 4,7 kHz (pico leve) | |
| Mod popular (50k vol / 250k / 10 nF) | corte sobe para ~14,6 kHz | é o "Stage modificado" |

Implicação: o "calor" de um Stage original é em boa parte **elétrico**, a carga de 10k sobre ~0,5 H, e não só do amp. Fica parâmetro "Harp L" + escolha "Original / Modded / Direct".

## 7. Fender Twin Reverb AA270 (parte superior do esquema lida)

| Estágio | Valores | Nível |
|---|---|---|
| Entrada | jacks 1/2 com 68k/68k e 1 MΩ; V1 7025, catodo 1500 Ω // 25 µF, placa +210 V | [P] esquema AA270 |
| Tonestack | treble 250 pF / 250k-A; bass 0,1 µF / 250k-A; mid 0,047 µF / 10k-A; slope 100k | [P] |
| Volume | 1M-A, bright 120 pF | [P] |
| Mixer | ½ 7025, retorno de reverb 3,3 M // 10 pF; acoplamento 0,047 / 220k | [P] |
| PI | 12AT7 long-tail, cauda 22k + 330k/330k, 270 Ω; NFB 820 Ω / 100 Ω | [P] |
| Potência | 4× 6L6GC, grade 1500 Ω, tela 470 Ω/1 W, placa ~+405–460 V, carga 4 Ω, 2×12 | [P] |
| **Reverb** | **só no canal Vibrato**. Envio por 500 pF → driver 12AT7 (V3, metades em paralelo, catodo 2200 Ω // 25 µF) → transformador TR4 → tanque → recovery ½ 7025 (V4B, 820 Ω // 25 µF; filtro de entrada 0,002 µF / 220k) → pot Reverb 100k-L via 0,003 µF → mistura 470k / 220k | [P] |
| **Vibrato** (na verdade tremolo) | V5 12AX7: oscilador por deslocamento de fase (0,01 / 0,02 / 0,02 µF, Speed 3M reverse-audio); a segunda metade acende a lâmpada do opto ("roach" lâmpada/LDR). O LDR desvia o sinal no nó de acoplamento do canal Vibrato (1M / 1M / 0,01 µF, 2,2 M); Intensity 50k reverse-audio | [P] |
| Fonte | retificador de silício (6 diodos) → 2× 70 µF/350 V em série (≈35 µF) // 220k; choke TR2; B+ +460 V (standby) / **+410 V tocando** → 20 µF → 10k / 22k → 20 µF | [P] → sag modesto, com o "afundar" do B+ em acordes fortes |
| Bias | fixo, −52 V da tomada de 3,3k / 50 µF, ajuste 10k-L (≈ −44 V na grade) | [P] |
| Implicação prática | no Twin, o Rhodes vai no **canal Vibrato**, o único com reverb e tremolo; o canal Normal é seco | [P] + uso histórico [S] |

## 8. Suitcase (segundo "amp")
Esquemas completos no manual cap. 11 (pré 80 W Peterson, pré 100 W MC1458 + LED/LDR, potência 2×50 W). Veja RESEARCH.md §1.4.

## 9. Ruídos e sujeiras (todos físicos)
- **Hum:** a polaridade alternada por grupo da harpa cancela a maior parte, e o residual depende da nota [H].
- **Barra de sustain sem aterramento:** acopla estática/RF nos captadores graves [S].
- **Hiss:** no Stage, a carga de 10k reduz o sinal; técnicos relatam "volume baixo e chiado" [S].
- **Crackle de pot:** só ao mover o controle.
- **Mecânica:** tine encostando no polo, double stroke, ruído de abafador.

---

## Pendências
1. Profundidade do Bass Boost: 14,6 dB (calculado) contra 18 dB (Gabrielli).
2. Esquema do Twin de 1973–75 com master volume: o arquivo "SF 100" está em baixa resolução, então o AA270 é a base.
3. Perfil de conicidade do tine (explica os agudos mais rígidos).
4. Massa e β do martelo do Rhodes (Falaize usa valores de piano acústico).

---

## 10. Implementação v0.1 (plugin) — o que foi calibrado e como

| Estágio | Arquivo | Decisão / calibração | Nível |
|---|---|---|---|
| Taxa interna | `dsp/resample.rs` | 2× (88,2/96 kHz) para captador e válvulas; meia-banda de 71 taps | [DESIGN] |
| Tines | `dsp/keys.rs` | comprimento da carta (suavizado), Ø 1,5 mm, massa da mola deduzida do comprimento; modos 0,48 / 1 (+gêmeo do garfo) / 7,1 / 20,4 / 39,7 / 62,9 / 93,1 + polarização horizontal | [MAN][ART][INFER] |
| Garfo | `dsp/keys.rs` | fundamental dividida em 62 % rápida (1,8× a taxa de Shear) + 38 % lenta (0,55×), 0,01–0,5 Hz de distância: decaimento "rápido, depois gradual" e batimento | [DESIGN←PAT 2,972,922] |
| Martelo | `dsp/voice.rs` | Hunt–Crossley, β = 2,3, λ = 0,45 s/m, K = 1,6·10¹⁰ × zona (1/1,8/3,2/5,6/9); contato resultante 0,3–2 ms | [DESIGN] |
| Participação | `dsp/keys.rs` `PART` | m2 0,0134 · m3 2,55 · m4–6 8,4 × φ₁: **calibrado para o F3 pp de Gabrielli fig. 5** (resultado: sub −36/−35, H2 −21/−15, H3 −23/−20, m2 −59/−52, m3 −40/−40, m4 −59/−60 dB) | [DESIGN←ART] |
| Velocidade | `dsp/engine.rs` | v₀ = (0,1 + 1,5·vel^p)·(f₀/196)^0,8 m/s, p = 3,5 − 2·Touch. F3 na ponta: vel 64 ≈ 1,04 mm (H2 −22 dB), 80 ≈ 1,6 mm (−16), 100 ≈ 2,6 mm (−8,4), 127 ≈ 4,5 mm (+1,7). Recalibrado duas vezes a pedido do usuário (p = 3 − 2·Touch na segunda versão): o latido forte fica para 100–127 | [DESIGN] |
| Captador | `dsp/pickup.rs` | tabela Φ(ρ, gap) gerada por `tools/gen_pickup_table.py`; Catmull–Rom; v = dΦ/dt; EMF 5,2·10⁻⁴ → ~0,12 V no jack para uma nota mf | [INFER] |
| Pedal e teclas seguras | `dsp/engine.rs` | cada golpe dá um empurrão pelo trilho (0,25 % do momento do martelo, decai com a distância na harpa) em todas as tines livres: com pedal, todas (o halo); sem pedal, só as teclas seguradas, cujo abafador também está levantado. F5 segurado recebe ≈ 0,007 mm de um F4 forte. Ressonância seletiva em frequência (oitava) não é modelada: as harmônicas do Rhodes nascem no captador, não no metal | [DESIGN←MAN cap. 2] |
| Harpa/Stage | `dsp/stage.rs` | MNA com indutor; testes conferem o shelf do Bass Boost e o passa-baixa de ~4 kHz | [SCH][ART] |
| Twin | `dsp/amp.rs` | 7025 por tabela de Koren; TMB + volume + bright por MNA; ganho do PI ajustado para que um acorde mf com Volume em 4 dê ~6–8 V no alto-falante (limpo) e ff ~13 V (compressão leve) | [SCH][INFER] |
| Potência | `dsp/amp.rs` | curva aberta tabelada, NFB β = 0,109 resolvida por Newton (2 passos), sag 11 %/A, ripple de 120 Hz com o Noise | [SCH][INFER] |
| Caixa/mic | `dsp/cab.rs` | biquads paramétricos (sem IR) | [DESIGN] |
| Limite de segurança | `dsp/engine.rs` | soft clip só acima de −1 dBFS | [DESIGN] |

**Ficou de fora na v0.1 (e o impacto audível):**
- Suitcase (pré, autopan estéreo, 4×12): sem som estéreo de vibrato. É o próximo passo.
- Double stroke no grave e tine encostando no polo: sem os "clacks" de piano desregulado.
- Ruído mecânico de teclas e abafadores: só o abafamento é modelado, não o "tchk".
- Saída mono com o Twin, a linha e o DI (L = R). Com o Leslie (v0.3) a saída é estéreo.
- Não linearidade geométrica do tine (pitch glide em ff): não modelada; magnitude desconhecida.

## 11. Leslie 147 (v0.3)

Ligação de época: Stage → pré "combo" (um estágio 12AX7 com controle de nível) → amplificador do 147 → divisor de 800 Hz → corneta rotativa em cima, alto-falante de 15" disparando para baixo num tambor rotativo → caixa de madeira → dois microfones. Substitui o Twin (o Leslie era o amplificador).

| Estágio | Onde | Modelo | Evidência |
|---|---|---|---|
| Pré + amplificador | `dsp/leslie.rs` `LesliePower` | 12AX7 por tabela de Koren; drive = 0,04 + 2·d³ (Drive 1–10); saída reaproveita a curva push-pull do Twin (tabela, NFB por Newton, sag). O 147 real usa 12AU7 como divisor de fase e 2× 6550 (40 W): mesma topologia, curva da válvula não refeita | [ART][DESIGN] |
| Divisor | `leslie.rs` | 800 Hz, Linkwitz–Riley de 4ª ordem (soma plana com rotores parados); a rede passiva do 147 não foi modelada peça a peça | [ART][DESIGN] |
| Corneta | `leslie.rs` | driver: passa-baixa 6 kHz, +2,5 dB em 2 kHz; boca a 0,15 m do eixo | [DESIGN] |
| Tambor | `leslie.rs` | passa-alta 45 Hz, ressonância da caixa +3 dB em 110 Hz; fonte efetiva a 0,12 m do eixo | [DESIGN] |
| Velocidades | `leslie.rs` | corneta 48 / 400 rpm, tambor 40 / 342 rpm (medição de John Fisher num 147, polia do meio) | [ART] |
| Aceleração | `leslie.rs` | tempo até assentar: corneta 1,1 s subindo / 1,5 s descendo, tambor 5,5 / 5,0 s (manual do 122H: 1,8/2,4 e 7/5,5 s; dono de 122XB: 0,9/1,0 e 5,5/4,8 s). Stop = motores desligados, rotores param por inércia | [MAN][ART] |
| Doppler, reflexos | `leslie.rs` | geometria, não LFO: cada fonte gira no seu círculo; para cada microfone, 4 caminhos (direto, imagens nas paredes laterais a ±0,27 m com ganho 0,38, parede de trás a 0,25 m com 0,30), cada um lido de uma linha de atraso no comprimento real em movimento (interpolação de Hermite), com atenuação 1/d. Smith, Serafin, Abel & Berners, *Doppler Simulation and the Leslie*, DAFx 2002 | [ART][DESIGN] |
| Diretividade | `leslie.rs` | corneta: 0,40 + 0,60·((1+cos α)/2)^1,5 e um passa-baixa por caminho de 1,8 kHz (de costas) a 9,3 kHz (de frente); tambor: 0,65 + 0,35·(1+cos α)/2 | [DESIGN] |
| Microfones | `leslie.rs` | a 0,62 m do eixo, abertos de 0° (mono) a 180° (Mic Width) | [DESIGN] |
| Roda de modulação | `lib.rs` | CC1 ≥ 64 = rápido, < 64 = lento; vale até o controle Rotor ser mexido | [DESIGN] |

**Medido (testes em `leslie.rs` e `tests.rs`):**
- Tom de 2 kHz pela corneta rápida: frequência oscila entre 1962 e 2037 Hz (previsto ±37 Hz = 2π·0,15 m·6,67 Hz / 343 m/s × 2 kHz).
- Após 1 s em Fast, a corneta já passou de 90 % da velocidade e o tambor ainda está abaixo de 60 %.
- Variação de amplitude num tom puro, rápido: corneta ~19–34 dB (inclui cancelamentos entre o direto e os reflexos), tambor ~9–11 dB. Correlação L/R da corneta: +0,37 a 120°, ≈ 0 a 180°; do tambor ≈ +0,93 (o grave é quase mono, como no real).
- Níveis: Chorale −19,4 dBFS RMS e Gospel Grit −19,4 (Output −15 dB) no acorde de teste, contra −19,0 do Factory.
- CPU: balada de 12 s a 33× tempo real com o Leslie, contra 28× com o Twin.
- Sem cliques na troca lento → rápido → lento (`tools/clicks.py`).

**Não modelado:** ruído do vento da corneta e do motor, o "clunk" do freio, variação de correia, a válvula 6550 com sua curva própria, o pré exato do combo (há vários modelos de época).

## 12. O grave (v0.4)

Queixa do usuário, comparando com as amostras oficiais de Mark I do Nord Stage 4 (EP1/EP2, via `~/code/fjord`): "o grave do Nord é mais definido; o nosso tem uma distorção, uma destruição das fases senoidais". Medido com `tools/bass_compare.py` (C1, E1, G1 a velocidade 100, saída direta) e uma escala C2–C6.

**Causa 1: o sub-fundamental sacudia a lâmina.** O espectro do nosso E1 tinha picos fora da série harmônica (56, 67, 108, 138, 190, 231 Hz), todos n·f₀ ± 26 Hz: intermodulação, no captador, entre os harmônicos e o modo sub-fundamental. Gabrielli mostra que esse modo é da barra de tom ("present in the tonebar only") e, nos modos medidos a laser passados pelo captador (`A2-sidebands.wav`), as bandas laterais dele ficam em −53 a −58 dB. As nossas estavam em −9 a −17 dB no E1. Como o captador quase não vê o movimento lento do sub sozinho, a linha do sub ficava baixa e a calibração pela fig. 5 não acusava; mas ele deslocava o ponto de trabalho do captador e modulava cada harmônico.
- `SUB_MASS` 3 → 20: no A2 com H2 −12 dB, bandas do sub em −55 a −60 dB (Gabrielli −53 a −58) [ART].
- Razão do sub: das medições (0,83 F1 → 0,58 A2/F3), não mais sorteada entre 0,30 e 0,86.
- Com o sub mais pesado a barra absorve menos do golpe: velocidade do martelo × 0,94 para manter a excursão calibrada do F3 (1,04 / 1,61 / 2,60 / 4,49 mm a 64/80/100/127, idêntica à v0.3). A H2 do F3 subiu 1–2 dB (−6,6 dB a 100), porque o que virava banda lateral volta para os harmônicos.

**Causa 2: o grave levava golpe demais.** A escala por registro (f₀/196)^0,8 travava em 0,45 abaixo do E2: do E2 para baixo todas as notas recebiam o mesmo golpe e as lâminas longas balançavam 10–12 mm (contando o sub). O manual limita o balanço ao escapement do grave, 6,35–9,5 mm. Piso agora 0,25 (`REG_MIN`): E1 a velocidade máxima, 5,0 mm [MAN].

**Causa 3: grave baixo demais.** Na saída direta, o Nord fica plano de C2 a C6 (+1 a −5 dB); o nosso C2 estava 10 dB abaixo do C4: a tensão é proporcional à velocidade da ponta, e o grave vibra devagar. No piano real o técnico acerta o captador nota a nota "de ouvido" para nivelar (Volume Adjustment, 1,6–3,2 mm [MAN cap. 4]). Distância por nota = Pickup × (f₀/F3)^0,7, piso 0,45, só abaixo do F3 (o controle Pickup passa a valer do F3 para cima) [DESIGN←MAN, calibrado contra o Nord].

| Medida (vel. 100, saída direta) | v0.3 | v0.4 | Nord EP2 |
|---|---|---|---|
| E1: H2 / H3 (dB re f₀) | −0,2 / +1,8 | −23 / −4,8 | −8,6 / −4,5 |
| E1: bandas laterais mais fortes | −9 a −17 dB | −32 a −40 dB | ~−39 dB |
| Nível C2 / G2 / C3 (re C4) | −10 / −7 / −5 dB | −2 / −2 / −2 dB | +1 / −2 / −2 dB |
| Excursão máx. da ponta, E1 / F3 | 9,9 / 6,7 mm | 5,0 / 5,2 mm | — |

Saída geral −1,6 dB (o grave ganhou corpo); presets de volta a −17,3…−20,1 dBFS RMS no acorde de teste (Factory −19,0).

**Pelo Twin:** a caixa aberta 2×12 (passa-alta ~72 Hz) e o transformador tiram a fundamental do E1 (41 Hz); a 3ª harmônica fica ~10 dB acima dela. É o Twin; quem quer a fundamental dos graves usa Line ou DI.
