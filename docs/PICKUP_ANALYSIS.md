# Análise offline do captador e dos tines (2026-11-17)

> Só análise (Python em `analysis/`), nenhum código de plugin. Reproduzível:
> `python3 analysis/fit_probe.py`, `analysis/fit_shear.py`, `analysis/dynamics.py`,
> `analysis/tines.py reference/manual/fig6-2.gif > bars.txt && analysis/tine_table.py bars.txt docs/data/tine_lengths.csv`.
> Separação: **medido** (dados de fonte primária) · **calculado** (modelo nosso) · **não confirmado**.

## 1. Comprimento dos tines (`docs/data/tine_lengths.csv`)

- **Fonte:** carta de corte do Service Manual (fig. 6-2). Medi o comprimento de cada barra em pixels, 82 barras para 88 teclas. As teclas 1–7 compartilham o comprimento da tecla 8 (o manual chama todas de "0").
- **Escala:** ajustada nas duas pontas pelos limites de Shear (157 mm e 18 mm). Sai 0,254 mm/px com offset de −0,8 mm, praticamente proporcional. Isso sugere que a carta é em escala real.
- **Validação (calculada):** uma viga de Euler-Bernoulli de aço com Ø 1,5 mm e esses comprimentos dá frequência "nua" (sem mola) dentro de **±5 % da nota real de A1 a A4**. Esse acordo independente confirma a escala.

| Região | f_nua / f_nota | Leitura |
|---|---|---|
| A0–D#1 (comprimento igual) | 1,56 → 1,10 | O grave é afinado só pela **massa da mola**, que precisa baixar muito a frequência. |
| A1–A4 | 0,94–1,04 | O tine quase "nu" já está na nota; a mola só faz o ajuste fino (±1,5 tom, segundo o manual). |
| A5–C8 | 0,77–0,87 | O tine fino de 1,5 mm ficaria grave demais. Os agudos devem ser **mais rígidos** (o alargamento da base pesa mais nos tines curtos). **Não confirmado:** falta o perfil real da conicidade. |

## 2. Campo do polo do captador

- **Medido** (patente US 4,040,321, Seth Lover/CBS-Fender, 1977, fig. 8 "prior art" = o polo padrão do Rhodes):
  - Curvas do campo transversal a 0,020"/0,040"/0,060" da face do polo.
  - Picos em ±0,2", com 40/35/31 G, e cauda de ~10–13 G a 0,6".
  - Ímã Alnico 5 de 0,5" × 0,1875".
  - Ponta do tine com Ø 0,060".
  - Amplitude estacionária de **~1/8" em C3**.
  - Ímã fraco "para não criar arrasto magnético no tine".
- **Calculado:** modelo de cargas magnéticas superficiais (no espírito de Pfeifle) para o polo tronco-cônico, com a geometria tirada do próprio desenho da patente (face plana r = 0,039", base r = 0,145", altura 0,127").
  - Sozinho, o modelo põe o pico perto da borda da face plana (0,05"), o que não bate com a patente.
  - Com o sensor da sonda **0,20" (≈5 mm) atrás da cota anotada**, o modelo reproduz as três curvas: picos em 0,19/0,20/0,21" e máximos de 40/34,4/29,8 G (a patente dá 40/35/31) e cauda de 11 G (a patente dá ~10–13).
  - Interpretação (**não confirmada**): o elemento Hall de 1975 ficava recuado dentro do encapsulamento da sonda. Consequência importante: no gap real do tine (0,5–3,2 mm), o campo é **muito mais concentrado** do que as curvas da patente sugerem.
  - O formato de cunha ("edge") ajusta pior que o cone.

## 3. Sinal do captador contra medições de Shear (G3, 196 Hz)

Testei três aproximações do fluxo: Bz na ponta do tine (Pfeifle), ∫Bz² ao longo do tine (vareta magnetizada axialmente, por reciprocidade) e |By| (o argumento da patente Lover). As tensões vêm de v ∝ dΦ/dt com a ponta em y = y0 + A·sin.

Alvos medidos (Shear, fig. 2.5):
- Tine deslocado 5 mm: H2 −15, H3 −29, H4 −30, H5 −44 dB.
- No eixo: H1 −19, **H2 0**, H3 −25, H4 −13, H5 −23, H6 −33 dB.

**Melhor ajuste (calculado):** a aproximação Bz na ponta, com **gap 3,2 mm, amplitude 3,0 mm**, resíduo de 0,2 mm fora do eixo no caso "no eixo" e offset de **5,25 mm** no caso "5 mm". Erro médio ≈ 3 dB por harmônico.

Quatro fontes independentes concordam:
- O gap de 3,2 mm é o limite superior do manual (1/8").
- A amplitude de 3 mm é a da patente Lover (1/8" em C3).
- O offset que ajusta é praticamente os 5 mm declarados por Shear.
- A geometria do polo veio do desenho da patente.

A aproximação ∫Bz² fica logo atrás (2,4 mm / 2 mm). **|By| é descartada.**

O caso "no eixo" real nunca é perfeito: os ímpares que Shear mediu (H1 −19) exigem ~0,2 mm de desalinhamento. Isso é um parâmetro de "Condition" natural, somado à 2ª polarização do tine.

## 4. Voicing (fig. 4-8 do manual) — calculado, gap 3,2 mm, A 3 mm

| Offset | H1 | H2 | H3 | H4 | Leitura |
|---|---|---|---|---|---|
| 0 mm | −90 | **0** | −90 | −13,5 | "Pure overtone (shallow)" ✔ |
| 0,5 mm | −11 | **0** | −15 | −14 | Ainda dominado pelo 2º harmônico |
| 1,0 mm | −4 | **0** | −8 | −17 | Rico |
| 1,5 mm | **0** | −1,5 | −5,5 | −27 | **"More overtone (ideal)"**: o mais rico ✔ |
| 3,0 mm | **0** | −30 | −12 | −20 | Indo para fundamental |
| 5,0 mm | **0** | −7,5 | −35 | −29 | "Deep" ✔ |

(Offsets ≥ 6 mm com A = 3 mm saem da grade de ±9 mm; resultado descartado.)

## 5. Dinâmica de toque (calculada, offset 1,5 mm)

| Gap | Nível de 0,25 → 4 mm de amplitude | Harmônicos/fundamental |
|---|---|---|
| 0,5 mm | +19,5 dB | −17 → **+12 dB** |
| 1,6 mm | +21,1 dB | −43 → +8 dB |
| 3,2 mm | +22,9 dB | −27 → +5 dB |

- **Bark confirmado no modelo:** tocar leve dá quase senoide; tocar forte deixa os harmônicos acima da fundamental. O nível cresce menos que a amplitude (16× = 24 dB lineares, saem ~20 dB), uma compressão leve que vem do próprio captador.
- **Volume contra gap:** de 3,2 mm para 0,5 mm ganha +22,5 dB de sensibilidade, o que confirma o "menor gap = maior volume" do manual.
- **Não confirmado:** o manual diz que gap menor dá "mais resposta dinâmica" (aumento percentual de volume com o toque). Em nível RMS puro, o modelo mostra o contrário (+19,5 contra +22,9 dB). Hipóteses:
  - (a) o efeito é de loudness percebido, porque os harmônicos crescem muito mais cedo com gap pequeno;
  - (b) o gap pequeno permite tines mais curtos ou golpes diferentes;
  - (c) a 2ª polarização ou o arco do tine, que ainda não estão no modelo.
  - Precisa do seu ouvido ou de outra fonte.

## 6. O que entra no plugin a partir daqui

1. **Tabela Φ(y, gap) pré-calculada** do polo cônico. Em tempo real é só uma busca 2D com interpolação, barata.
2. **Parâmetros por nota:**
   - Timbre (offset): padrão ~1–1,5 mm acima do eixo, com variação aleatória.
   - Volume (gap): 1,6–3,2 mm; 0,5 mm no médio/agudo a partir de 1972.
   - Resíduo de alinhamento.
3. **Amplitude da ponta do tine** em mm, vinda do modelo modal (calibração: ~3 mm em C3 estacionário).
4. A 2ª polarização e o arco da ponta ficam como próximos refinamentos.
