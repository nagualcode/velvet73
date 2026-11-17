# velvet73 — Pesquisa de base (Rhodes Mark I + amplificador)

> Fase 1 da metodologia Jardim Sonoro: pesquisa antes de código. **Nenhum código escrito.**
> v3 — 2026-11-17. Fontes primárias baixadas em `reference/` (no `.gitignore`):
> Rhodes Service Manual completo (cap. 1–11 + figuras), papers de Falaize & Hélie (IRCAM),
> Shear (tese + NIME 2012), Pfeifle (DAFx-17), Sønderbo (AAU 2024), guia de medição de captador,
> **patentes** US 2,972,922 · 3,270,608 · 3,384,699 · 3,418,417 · 3,644,656 (Harold Rhodes / Leo Fender)
> e US 4,040,321 (Seth Lover, captador), **esquemas do Twin Reverb** silverface AA769, AA270, 100 W e 135 W.
> Análise offline: `docs/PICKUP_ANALYSIS.md` e `docs/data/tine_lengths.csv`.
>
> Níveis de evidência:
> **[P]** primária (manual Rhodes, esquemático, paper com medição) ·
> **[S]** secundária séria (técnicos, fóruns de restauração) ·
> **[D]** dedução nossa, a verificar ·
> **[?]** contraditório ou não encontrado

## Decisões do usuário (2026-11-17)

1. **Recorte aprovado:** Stage 73 Mark I (~1973–75) → Fender Twin Reverb silverface primeiro; Suitcase em seguida; DI como referência.
2. **Sem acesso a um Mark I real por ora** → calibração por papers + medições publicadas + ouvido do usuário.
3. **Pode citar Rhodes/Fender** no README e no site (uso nominativo; sem logos nem trade dress).

---

## 0. Correções em relação à v1

| v1 dizia | Fonte primária diz | Fonte |
|---|---|---|
| Tone bar de latão | Fábrica: **"twisted steel bar"** (manual cap. 7). A patente US 3,644,656 (Fender & Rhodes, 1970) explica a torção: a barra de inércia tem seção **oblonga alongada no plano de vibração do tine**, o que **suprime os sobretons da própria barra** (problema maior nos graves); um trecho é deslocado para dar espaço à excursão do tine. A US 3,418,417 descreve uma variante em **alumínio extrudado com contrapeso de aço**. Pfeifle diz "latão". Material variou por geração. | [P] manual; [P] patentes; [P] Pfeifle → **[?] material**, forma resolvida |
| Captadores sem cancelamento de hum | Grupos com **polaridade alternada** para reduzir interferência (Shear); Pfeifle diz que **cada bobina tem duas seções em contrafase** (humbucking). | [P] Shear 2011; [P] Pfeifle 2017 |
| Indutância/impedância da harpa desconhecida | Após a mudança de fiação (saída ×4): **impedância de saída ≈ 2500 Ω**. | [P] manual cap. 10 |
| Martelo de madeira até ~1976/77 | Shank e flange moldados em **Cycolac (ABS)** já na "3ª geração" (antes de set/1975); cabeça de bordo com ponta trocável. Flanges com 15 % de teflon na versão pós-1975. | [P] manual cap. 2 e 8 |

---

## 1. Dados primários do Service Manual (os números que o modelo vai usar)

### 1.1 Tone source
- Diapasão de **braços desiguais** (patente US 2,972,922). O tine é o braço "mais resiliente"; a tone bar vibra na mesma frequência. Apertar a tone bar com os dedos **mata a nota na hora** [P cap. 1].
- A tone bar **tolera o tine desafinado de F a A em torno de G** "sem perda apreciável de suporte" → acoplamento largo, não um ressonador estreito [P cap. 1]. Implicação de modelo: tone bar com Q baixo/médio e acoplamento forte [D].
- Mola de afinação = contrapeso: para fora → grave, para dentro → agudo; faixa de **±1,5 tom** [P cap. 5].
- Tines: todos com a **mesma forma, variando só o comprimento**; kit de reposição de 111 mm cortado com alicate segundo a carta de corte (fig. 6-2, em escala relativa) [P cap. 6]. Shear: **18–157 mm** no 88 teclas, **Ø 1,5 mm** com alargamento na base; fundamentais **41 Hz–2,6 kHz** no 73 [P].
- Evolução do tine: fio de piano Ø 1,905 mm (40 mil golpes) → afinado por retífica sem centro (1,5 milhão) → **forjado/swaged** (6+ milhões) [P cap. 7].
- Q medidos num Mark I 88 de 1974 (Shear, tabela 2.1) [P]:

| Nota | f0 (Hz) | Q | τ (37 %) aprox. = 2Q/(2πf0) |
|---|---|---|---|
| E♭2 | 77,8 | 949 | 3,9 s |
| E♭3 | 155,6 | 731 | 1,5 s |
| E♭4 | 311,1 | 1520 | 1,6 s |
| E♭5 | 622,3 | 2175 | 1,1 s |
| E♭6 | 1244,5 | 1761 | 0,45 s |

### 1.2 Ação e martelos
- Key dip 9,5 mm ±0,8 [P cap. 4].
- **Escapement** (folga martelo–tine com a tecla no fundo): grave extremo 6,35–9,5 mm; tone bar 41: 1,6–3,2 mm; agudo extremo 0,8–2,4 mm [P fig. 4-2]. Escapement pequeno demais no grave → **"double stroking"** (golpe duplo) com som perturbador; zero → o martelo abafa a nota se o dedo segura [P cap. 10].
- **Pontas de neoprene** (desde início de 1971) graduadas em dureza e altura [P cap. 4]:

| Martelos (numeração 88) | Durometer | Altura |
|---|---|---|
| 1–30 | 30 | 6,35 mm |
| 31–40 | 50 | 7,94 mm |
| 41–50 | 70 | 9,5 mm |
| 51–64 | 90 | 11,1 mm |
| 65–88 | "wrapped" (extra duro) | 11,1 mm |

  O manual reconhece a **mudança brusca de timbre** entre zonas e diz que músicos estendem a de 90 até o martelo 67 ou além [P]. Isso é uma "sujeira" de fábrica real: degraus de timbre no teclado.
- Felt sulcado: "as paredes do sulco agem como abafadores" → ataque e sustain pioram [P cap. 8]. O neoprene resolveu.
- **Striking line:** 57 mm da borda do tone generator no grave extremo até ~3 mm no agudo extremo; curva da trilho da tone bar achada "por tentativa dolorosa" [P cap. 4].
- **Abafadores:** módulos de 12, três tamanhos (grave: feltros longos e largos; médio; agudo: curtos e estreitos); folga 9,5–12,7 mm com tecla abaixada. A barra de pedal levanta **todos** e o manual descreve explicitamente a **ressonância simpática** ("como num piano acústico") [P cap. 2].

### 1.3 Captador, timbre e volume
- **Timbre (fig. 4-8):** o polo do captador é um **cone** apontado para a ponta do tine; o tine vibra **atravessando o eixo**. Ponta muito acima do eixo = "pure fundamental (deep)"; levemente acima = **"more overtone (ideal)" — ajuste de fábrica**; no eixo = "pure overtone (shallow)" (2º harmônico domina) [P cap. 4; confirmado por Shear figs. 2.4–2.5].
- **Volume (fig. 4-9):** gap tine–captador de **1,6 a 3,2 mm**; pianos após **março de 1972** aceitam **0,5 mm** no médio/agudo. "Quanto menor o gap, maior o volume e, mais importante, mais pronunciada a resposta dinâmica" [P cap. 4].
- **Fiação:** original = grupos de ~12–13 em paralelo ligados em série (fig. 10-3); modificada = **grupos de 3 (primeiro de 4) em paralelo, em série**, saída ×4, **≈ 2500 Ω** [P cap. 10, figs. 10-3/10-4]. Cada captador ≈ 180 Ω DC [S].
- **Afinação:** fábrica em temperamento igual; o manual traz **carta de stretch** (A0 −20 ¢ … C8 +30 ¢) [P fig. 5-4]. Dá um parâmetro "Stretch" autêntico.

### 1.4 Suitcase (cap. 11)
| Sistema | Pré | Vibrato | Potência |
|---|---|---|---|
| **80 W (Peterson)** | Discreto, transistores **2N3392** selecionados; tonestack passivo treble/bass (pots lineares 100k) [P fig. 11-8] | Oscilador com 2N3053 acendendo duas **lâmpadas #19** alternadamente (via diodos), cada uma diante de uma fotocélula **CdS ORP61** → autopan estéreo. Intensity e Speed em pots concêntricos 50k log reverso [P] | Módulo de potência + regulador Peterson [P figs. 11-9/10] |
| **100 W (Haigler) / Janus I** | Op-amps **MC1458**: entrada ganho ≈ 22 (220k/10k, 120 pF), volume 5k log, **Baxandall** bass/treble 50k lin [P fig. 11-1] | MC1458 como oscilador (onda triangular/quadrada) → Q1 2N3392 aciona **LED + LDR** (dois optos) em contrafase; Speed 100k; Intensity 25k [P] | **2× 50 W classe AB** quase-complementar (2N5781/RCA29B/RCA30B/2N6254), bias por cadeia de diodos + trimpot 500 Ω, trilhos **±30 V não regulados** com 11 000 µF [P fig. 11-3] |

Sujeiras documentadas: lâmpada #19 queima (falha nº1 do 80 W) [S]; **R3/R5 = 820 Ω** no amp de 100 W, que técnicos trocam por 680 Ω contra **distorção de crossover** [P esquema + S]; recap errado do vibrato → "thump" [S]. O MC1458 tem slew rate baixo (~0,5 V/µs) e ruído alto para padrões atuais [D: datasheet genérico] → leve suavização de transientes no pré do 100 W.

**Stage passivo:** o manual não traz o esquema dos controles Volume/Bass Boost do Stage → **[?]** levantar com técnicos/fóruns ou medir quando houver um piano.

### 1.5 Patentes (o que resolvem)
| Patente | Conteúdo útil para o modelo |
|---|---|
| **US 2,972,922** (H. Rhodes, 1961) | Objetivo declarado: "forte ataque percussivo seguido de decaimento **primeiro rápido e depois gradual**" e "forte preponderância da fundamental", com sobretons de palheta "fracos e discretos". Tine de aço carbono (corda de piano). A barra sozinha, golpeada, quase não soa. Massa adicional é preferível a alongar a barra (que gera sobretons). → **Decaimento em dois estágios** é alvo de projeto, não acidente. |
| **US 3,270,608** (H. Rhodes, 1966) | Mecânica da ação (martelo, abafador por correia). |
| **US 3,384,699** (H. Rhodes, 1968) | Tone generator suspenso em **molas** entre apoio e base, "fracas o bastante para permitir vibração sustentada" e com almofada macia. A mola mais distante **pode comprimir num golpe forte** → compliance de montagem que depende da dinâmica. O ajuste de altura muda **timbre, ponto de golpe e escapement juntos**. |
| **US 3,418,417** (H. Rhodes, 1968) | Diapasão multicomponente: barra de alumínio + contrapeso de aço + travessa de aço temperado com o tine prensado. Afinação por mola helicoidal. |
| **US 3,644,656** (C. L. Fender & H. Rhodes, 1972) | Barra de inércia **torcida** e oblonga para suprimir os próprios sobretons; ressonância mútua dos dois braços pode **reforçar certos sobretons** em algumas combinações de comprimento e seção. |
| **US 4,040,321** (Seth Lover, CBS, 1977) | **Curvas medidas de campo do polo padrão Rhodes** ("prior art", fig. 8). Explica o "frequency doubling" (gradiente do campo inverte no eixo). Posição de repouso usual ~0,15" do eixo. Amplitude estacionária ~1/8" em C3. Ímã Alnico 5 0,5"×0,1875", propositalmente fraco para evitar arrasto magnético. Ponta do tine Ø 0,060". Golpe ajustado para excitar "substancialmente só o modo fundamental". Propõe um polo bifurcado mais "pianístico". **[?]** não sabemos se entrou em produção. |

### 1.6 Fender Twin Reverb "AA270" (esquema lido; alvo de 1973–75)
- **Entradas:** 7025 (12AX7 baixo ruído), 68k/68k + 1 MΩ, catodo 1500 Ω // 25 µF, placa +210 V.
- **Tonestack TMB:** treble 250 pF, 250k-A; bass 0,1 µF, 250k-A; mid 0,047 µF, 10k-A; slope 100k. **Volume 1M-A com bright de 120 pF.**
- **Mistura:** ½ 7025 (V4A) somando canais (220k / 3,3M // 10 pF no retorno do reverb).
- **PI:** 12AT7 long-tail (cauda 22k + 330k/330k, 270 Ω), NFB 820 Ω / 100 Ω do secundário.
- **Potência:** **4× 6L6GC** a +405–460 V, resistores de grade 1500 Ω, de tela 470 Ω/1 W. Carga total 4 Ω (2×12). Fonte com choke e retificador de silício.
- **Ainda não lido:** parte inferior do esquema (reverb driver/recovery, vibrato óptico 12AX7, bias). Esquemas de **100 W com master volume** e **135 W ultralinear (1977+)** baixados, mas em baixa resolução.

---

## 2. A cadeia física, estágio por estágio

```
tecla → martelo (ABS/bordo + neoprene graduado) → TINE (aço swaged) ⇄ TONE BAR (aço torcido)
                                                    │  diapasão desigual, acoplamento largo
                                    ponta do tine atravessa o eixo do POLO CÔNICO
                                                    │  B(x,z) não-linear 2D; v = −N·dΦ/dt
                       harpa: 73 captadores, grupos de 3 em paralelo, em série (≈2,5 kΩ, indutiva)
                                                    │
                    Stage: Volume + Bass Boost passivos   |  Suitcase: pré 80/100 W + autopan + amp
                                                    │
                       cabo (C) → 1 MΩ do Twin → 12AX7 → TMB → reverb/vibrato
                       → PI 12AT7 → 4×6L6 → transformador → 2×12 → microfone → sala
```

### 2.1 Mecânica

| Componente | Função projetada | Efeito no som | Sujeira / efeito não intencional |
|---|---|---|---|
| Tecla + pedestal (feltro) | Transmite o toque. | Curva velocidade→força. | "Thock"/"clack" mecânicos; key pedestal mudou em jun/1978 [P]. |
| Martelo + ponta | Massa e mola de contato. | Contato Hertziano com histerese (Stulov/Hunt-Crossley): forte = contato curto = mais parciais. | Degraus de timbre entre zonas de dureza [P]; ponta gasta/sulcada [P]; double stroke no grave com escapement curto [P]. |
| Escapement | "Free throw" do martelo. | Sensibilidade do toque. | Zero → martelo abafa a nota; pouco → golpe duplo [P]. |
| Striking line | Ponto do golpe. | Filtra modos do tine. | Tines Torrington e Singer com strike points diferentes no mesmo piano de 1976 [S]. |
| Tine | Viga em balanço com massa (mola) perto da ponta. | Parciais inarmônicos no ataque, decaem rápido; depois quase senoidal [P Shear fig. 2.2]. Gabrielli et al. (JASA 2020) analisam esses parciais especificamente [P, a obter]. | Grande deflexão relativa ao diâmetro → efeitos não-lineares (tipo Kirchhoff) e **duas polarizações** (vertical domina; horizontal vem de acoplamento ou imperfeição da ponta) [P Pfeifle, medido com câmera]. Fadiga → desafinação de notas isoladas [P cap. 5]. |
| Tone bar | Segundo braço; guarda energia. | Sustain longo (Q 700–2200) [P]. | Tolerância larga de afinação; se muito longe, sustain curto e batimento [D]. |
| Parafusos, molas e gromets | Fixação e altura (escapement). | Amortecimento e vazamento para o trilho. | Mudar a altura desregula timbre, volume e abafador [P]. |
| Abafadores | Calam ao soltar. | Release com "tchk". | Tensão por "leaf spring" de alumínio dobrável; abafador fraco = rabo longo [P]. |
| Pedal | Levanta todos os abafadores. | **Ressonância simpática** [P cap. 2]. | Folga na vareta = perda de sensibilidade do pedal [P]. |

### 2.2 Captador e harpa
- Modelo de Falaize & Hélie (IRCAM): Φ ∝ 1/(q+lp)², viga **sobre** o captador, uma polarização. É passivo e elegante, mas **não captura o voicing** (deslocamento lateral em relação ao eixo do cone) [P, leitura nossa].
- Modelo de Pfeifle (DAFx-17, a partir de FEM em Münster/Pfeifle 2014/2017): "monopolos magnéticos" distribuídos sobre a **geometria idealizada do polo cônico**, campo integrado em 3 partes; a ponta do tine é um ponto que percorre um arco de círculo centrado na fixação, nas duas polarizações [P]. Este **captura o voicing** e foi validado contra gravações (Gabrielli 2020, segundo Sønderbo) [S].
- Sønderbo (2024) usou filtros no lugar do captador e conclui que é aí que o timbre se perde: "o captador é o mais importante para o som do Rhodes" [P].
- Harpa: série/paralelo, ≈2,5 kΩ, hum-cancelling por seções em contrafase [P]. A indutância não foi encontrada [?]; com 2,5 kΩ e o cabo forma passa-baixa ressonante (análogo a captador de guitarra) [D].

### 2.3 Twin Reverb silverface
Conhecimento padrão [S/D]; o esquema exato (AB763 → AA769 → master volume/ultralinear de 1977+) ainda **não** foi baixado.
Estágios: entrada 1 MΩ / 68k → 12AX7 com bright cap → **TMB** (vale nos médios) → reverb de molas (driver 12AT7, tanque Accutronics) → vibrato óptico ("roach" LDR) → PI 12AT7 → 4×6L6 → transformador → 2×12.
Sujeiras: saturação suave nos graves fortes, sag, compressão de potência, crossover leve, saturação do transformador, breakup do cone, hum 60/120 Hz, hiss, crash das molas.

---

## 3. O que mudava o som de propósito vs por acidente

**Intencional:** diapasão desigual, neoprene graduado, timbre (altura do tine), volume/dinâmica (gap), escapement por região, striking line curva, fiação ×4, hum-cancelling, stretch tuning opcional, vibrato estéreo, amp e reverb.

**Acidental (as sujeiras):**
1. Não-linearidade do captador dependente da amplitude: forte = bark; o timbre limpa enquanto a nota decai.
2. Degraus de timbre entre zonas de dureza das pontas [P].
3. Variação nota a nota de timbre, gap, escapement e striking line (tudo ajustado "de ouvido", segundo o próprio manual).
4. Segunda polarização do tine → batimentos lentos [P].
5. Parciais inarmônicos de ataque; deflexão grande → leve não-linearidade geométrica [P].
6. Fadiga de tines, pontas sulcadas, gromets ressecados.
7. Double stroke no grave; martelo abafando a nota com escapement zero [P].
8. Ressonância simpática com o pedal [P]; ruídos de abafador, tecla e pedal.
9. Ressonância harpa+cabo+carga; hum residual; crackle de pot.
10. Suitcase: inércia térmica da lâmpada #19 ou LDR no autopan, crossover do classe AB, slew do MC1458, sag dos trilhos não regulados.
11. Twin: compressão, sag, transformador, falante, molas.

---

## 4. Arquitetura de modelagem proposta (sem código)

Por tecla, 73 vozes fixas (uma por tine, sem roubo de voz):

1. **Martelo:** massa + contato Hertz/Hunt-Crossley com histerese; k e α nas cinco zonas de dureza da tabela do manual; altura da ponta e escapement por região → velocidade de impacto efetiva, double stroke no grave e martelo abafando.
2. **Tine:** síntese modal (6–8 modos) de viga em balanço com massa pontual da mola; comprimentos derivados da carta de corte (escala relativa) + 18–157 mm de Shear; golpe na striking line real. **Duas polarizações** com leve desintonia. Termo não-linear geométrico opcional (barato se modal).
3. **Tone bar:** 1–2 modos acoplados largamente; Q calibrado pela tabela de Shear (interpolada).
4. **Captador:** modelo 2D tipo Pfeifle (polo cônico + ponta do tine em arco). Parâmetros por nota: offset do eixo (Timbre), gap (Volume/dinâmica). Pré-calcular a **tabela Φ(x, z)** offline por geometria → lookup 2D barato em tempo real. Colisão macia se o tine encostar no polo.
5. **Abafador/pedal:** amortecimento extra e ruído de contato; pedal libera tudo → simpatia via barramento fraco do trilho.
6. **Harpa → Stage → cabo → entrada:** fonte ≈2,5 kΩ + L estimada + C do cabo + 1 MΩ (WDF). Volume e Bass Boost do Stage pendentes de esquema [?].
7. **Twin:** triodos via WDF/DK, TMB em forma fechada, molas dispersivas, tremolo LDR, PI + push-pull com sag, transformador, falante/gabinete/microfone paramétricos.
8. **Suitcase:** pré 80 W discreto ou 100 W com MC1458, a partir dos esquemas; autopan com lâmpada (filtro térmico) ou LED/LDR; classe AB com crossover ajustável (o "820 vs 680 Ω" vira um parâmetro de condição).
9. **Ruídos:** hum (com o cancelamento parcial da harpa), hiss, crackle só ao mover controle, ruídos mecânicos sintetizados.
10. **Instância de piano:** semente de tolerâncias por nota (afinação, timbre, gap, escapement, desgaste, Q). Controles de época/condição: Raymac/Torrington/Singer, felt (pré-1971) vs neoprene, fiação original vs ×4, stretch on/off.

CPU: 73 × ~10 modos é barato; captador por lookup; não-linearidades só em vozes acordadas; sono em repouso; oversampling só no captador e no amp.

---

## 5. Até onde podemos chegar

| Com rigor (fonte primária) | Heurística calibrada | Fora de alcance sem hardware |
|---|---|---|
| Física do martelo, tine, tone bar e captador 2D (papers do IRCAM, Pfeifle, Shear). | Comprimentos absolutos dos tines (carta em escala relativa + limites de Shear); parâmetros do neoprene por durometer; indutância da harpa; esquema do Stage passivo. | Calibrar contra **um** Mark I específico. |
| Faixas de ajuste do manual (escapement, gap, timbre, striking line, durometer, impedância, stretch). | Twin: falante, gabinete e microfone sem IR medida. | Microfonia real de válvula, ar de uma sala específica. |
| Suitcase 80 W e 100 W componente a componente (esquemas completos). | Proporção de cada sujeira, validada por espectro e pelo seu ouvido. | — |
| Q por região (medido por Shear). | | |

---

## 6. Próximos passos de pesquisa (antes de codar)

Feito em 2026-11-17: patentes, esquemas do Twin, tabela de tines e análise do captador (ver `docs/PICKUP_ANALYSIS.md`).
Feito em 2026-11-17 (2ª rodada): Gabrielli 2020 + áudios, Falaize JSV, esquema do nameboard do Stage (EP-Forum), estimativa heurística da indutância da harpa, tabela modal (`docs/data/modes.csv`), rede do Stage (`analysis/stage_network.py`), reverb/vibrato/fonte do AA270. **Registro consolidado em `docs/CIRCUIT.md`.** Os itens 1–3 e 5 abaixo foram resolvidos lá.

Pendente:
1. **Papers pagos / não achados livres:** Gabrielli et al., JASA 148(5):3052 (2020), DOI 10.1121/10.0002002 (modos do diapasão por vibrometria laser; o mais importante que falta); Münster, Pfeifle et al., JASA 136:2164 (2014), resumo; Pfeifle & Münster (2017), capítulo de livro Springer; Falaize & Hélie JSV 2017 (versão aberta em HAL hal-01470949 — baixar).
2. **Esquema dos controles do Stage** (Volume / Bass Boost) — não está no manual; procurar em fóruns de restauração e em esquemas de "Stage piano harp".
3. **Indutância da harpa:** estimar a partir do Alnico 5 + bobina de ~180 Ω, ou achar medição publicada.
4. **Modos da tone bar** e acoplamento (decaimento em dois estágios) — calibrar com os Q de Shear e, se possível, Gabrielli.
5. Ler a parte inferior do AA270 (reverb, vibrato, bias).
6. Escrever `docs/CIRCUIT.md` com a cadeia final e os parâmetros por estágio.

---

## Fontes

**Primárias (baixadas em `reference/`)**
- Rhodes Service Manual, cap. 1–11: https://fenderrhodes.com/org/manual/toc.html (esquemas cap. 11: https://fenderrhodes.com/org/manual/ch11.html)
- Falaize & Hélie, "Guaranteed-passive simulation of an electro-mechanical piano: a port-Hamiltonian approach", DAFx-15 (IRCAM): https://www.ntnu.edu/documents/1001201110/1266017954/DAFx-15_submission_33.pdf/a87ee400-f47c-4938-92e7-4030b1338cb7 · projeto: https://afalaize.github.io/posts/rhodes
- Pfeifle, "Real-time physical model of a Wurlitzer and Rhodes electric piano", DAFx-17: https://dafx.de/paper-archive/2017/papers/DAFx17_paper_79.pdf
- Shear, tese de mestrado UCSB 2011: https://www.mat.ucsb.edu/Masters/GregShearMasters2011_12_5.pdf · Shear & Wright NIME 2012: https://vhosts.eecs.umich.edu/nime2012/Proceedings/papers/284_Final_Manuscript.pdf · NIME 2011: https://zenodo.org/records/1178161/files/nime2011_014.pdf (download bloqueado)
- Sønderbo, tese AAU 2024: https://projekter.aau.dk/projekter/files/719175589/Master_Thesis_Real_time_physical_model_of_the_Rhodes_Electric_Piano_Tobias_Sonderbo.pdf

**Secundárias**
- Amps do Suitcase: https://www.fenderrhodes.com/service/amp-diffs.html · https://fenderrhodes.com/service/amp-failures.html · https://www.vintagevibe.com/products/rhodes-5-pin-100w-suitcase-pre-amplifier-tremolo-ldr · https://www.diyaudio.com/community/goto/post?id=3491114
- Captadores: https://www.fenderrhodes.com/service/pickups.html · https://www.vintagevibe.com/blogs/news/history-of-rhodes-pickups · https://shadetreekeys.com/2011/10/01/parallel-wiring · https://cdn.shopify.com/s/files/1/0666/2821/files/Rhodes_Pickup_Measurement_Troubleshooting_-_REV1_27FEB2020.pdf
- Pontas: https://www.fenderrhodes.com/service/hammer-tips.html · https://www.vintagevibe.com/blogs/news/the-definitive-guide-to-vintage-vibe-hammer-tips · https://fenderrhodes.com/service/dyno-tips.html
- Linha do tempo e tines: https://fenderrhodes.com/history/timeline.html · https://chicagoelectricpiano.com/rhodes/the-ultimate-fender-rhodes-timeline/ · https://retrolinear.com/mk-i-torrington-tines.html
- Bark e voicing: https://chicagoelectricpiano.com/rhodes/fender-rhodes-bark/ · https://www.vintagevibe.com/pages/vintage-vibe-variable-voice-control
