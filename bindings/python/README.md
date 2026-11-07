# Synthesis theory: Python bindings

Python bindings for [synthesis-theory](https://github.com/chethandvg/synthesis-theory), built from the Rust core with PyO3 and maturin. Function names and results are identical to the Rust crate; signals come back as float64 numpy arrays, and the instruments take the argument names and defaults of the earlier `lifesound.py` toolkit. The shared golden vectors check it in CI.

```bash
pip install "git+https://github.com/chethandvg/synthesis-theory@v0.1.0#subdirectory=bindings/python"
```

```python
import synthesis_theory as st

st.karplus_strong_pitch(440.0)  # 438.8059701492537: the string sounds 4.70 cents flat
tuned = st.karplus_strong_tuned(440.0, 2.0)  # the same pluck, tuned with an allpass
```
