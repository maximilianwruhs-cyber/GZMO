---
type: source
title: drive-research-llmlingua-cpu-leistung-und-leistungstests
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# drive-research-llmlingua-cpu-leistung-und-leistungstests

Ingested source summary (2026-06-08).

## Entities
- [Structured Prompt Compression](/entities/structured-prompt-compression.md) (CONCEPT)
- [tiktoken](/entities/tiktoken.md) (TOOL)
- [microsoft/llmlingua-2-bert-base-multilingual-cased-meetingbank](/entities/microsoft-llmlingua-2-bert-base-multilingual-cased-meetingbank.md) (SYSTEM)
- [mBERT Base (Multilingual)](/entities/mbert-base-multilingual.md) (SYSTEM)
- [microsoft/llmlingua-2-xlm-roberta-large-meetingbank](/entities/microsoft-llmlingua-2-xlm-roberta-large-meetingbank.md) (SYSTEM)
- [Position Bias](/entities/position-bias.md) (CONCEPT)
- [TensorFlow.js](/entities/tensorflow-js.md) (TOOL)
- [LLMLinguaCompressor](/entities/llmlinguacompressor.md) (TOOL)
- [Claude](/entities/claude.md) (SYSTEM)
- [GPT-2 Small](/entities/gpt-2-small.md) (SYSTEM)
- [ContextualCompressionRetriever](/entities/contextualcompressionretriever.md) (TOOL)
- [Retrieval-Augmented Generation (RAG)](/entities/retrieval-augmented-generation-rag.md) (CONCEPT)
- [Hermes](/entities/hermes.md) (SYSTEM)
- [nltk](/entities/nltk.md) (TOOL)
- [LLMLingua-2-small](/entities/llmlingua-2-small.md) (TOOL)
- [Multi-Turn-Agenten-Frameworks](/entities/multi-turn-agenten-frameworks.md) (CONCEPT)
- [@huggingface/transformers](/entities/huggingface-transformers.md) (TOOL)
- [Causal Perplexity](/entities/causal-perplexity.md) (CONCEPT)
- [Small Language Model (SLM)](/entities/small-language-model-slm.md) (CONCEPT)
- [BoL (Bag-of-Words/Lines) Checkpoint-Summary](/entities/bol-bag-of-words-lines-checkpoint-summary.md) (CONCEPT)
- [JavaScript/TypeScript](/entities/javascript-typescript.md) (CONCEPT)
- [Transformer Encoder](/entities/transformer-encoder.md) (SYSTEM)
- [Lost in the Middle](/entities/lost-in-the-middle.md) (CONCEPT)
- [Apple M1 Pro](/entities/apple-m1-pro.md) (SYSTEM)
- [Python Dependencies](/entities/python-dependencies.md) (CONCEPT)
- [GPT-4](/entities/gpt-4.md) (SYSTEM)
- [Microsoft Research](/entities/microsoft-research.md) (ORGANIZATION)
- [Grammatical Decay](/entities/grammatical-decay.md) (CONCEPT)
- [LLaMA](/entities/llama.md) (SYSTEM)
- [Data Distillation](/entities/data-distillation.md) (CONCEPT)
- [CPU](/entities/cpu.md) (SYSTEM)
- [@atjsh/llmlingua-2](/entities/atjsh-llmlingua-2.md) (TOOL)
- [Safetensors](/entities/safetensors.md) (CONCEPT)
- [Task-Agnostic](/entities/task-agnostic.md) (CONCEPT)
- [In-Context Learning (ICL)](/entities/in-context-learning-icl.md) (CONCEPT)
- [ONNX](/entities/onnx.md) (TOOL)
- [Prompt Compression in the Wild](/entities/prompt-compression-in-the-wild.md) (PROJECT)
- [LongLLMLingua](/entities/longllmlingua.md) (TOOL)
- [Information Entropy](/entities/information-entropy.md) (CONCEPT)
- [numpy](/entities/numpy.md) (TOOL)
- [OpenVINO](/entities/openvino.md) (TOOL)
- [accelerate](/entities/accelerate.md) (TOOL)
- [LangChain](/entities/langchain.md) (TOOL)
- [torch](/entities/torch.md) (TOOL)
- [Chain-of-Thought (CoT) Inferenzen](/entities/chain-of-thought-cot-inferenzen.md) (CONCEPT)
- [LlamaIndex](/entities/llamaindex.md) (TOOL)
- [transformers](/entities/transformers.md) (TOOL)
- [PyTorch](/entities/pytorch.md) (TOOL)

## Relations
- LLMLingua-2-small → RELATED_TO → Hermes
- LLMLingua-2-small → USES → Structured Prompt Compression
- Hermes → USES → BoL (Bag-of-Words/Lines) Checkpoint-Summary
- Hermes → RELATED_TO → LLMLingua-2-small
- Structured Prompt Compression → RELATED_TO → Retrieval-Augmented Generation (RAG)
- Structured Prompt Compression → RELATED_TO → Chain-of-Thought (CoT) Inferenzen
- Structured Prompt Compression → RELATED_TO → In-Context Learning (ICL)
- Structured Prompt Compression → RELATED_TO → Multi-Turn-Agenten-Frameworks
- Information Entropy → RELATED_TO → LLMLingua-2-small
- Causal Perplexity → RELATED_TO → LLMLingua-2-small
- Small Language Model (SLM) → USES → Causal Perplexity
- LongLLMLingua → RELATED_TO → LLMLingua-2-small
- LongLLMLingua → RELATED_TO → Lost in the Middle
- LongLLMLingua → USES → Position Bias
- LLMLingua-2-small → USES → Data Distillation
- LLMLingua-2-small → USES → Transformer Encoder
- Transformer Encoder → RELATED_TO → microsoft/llmlingua-2-bert-base-multilingual-cased-meetingbank
- LLMLingua-2-small → RELATED_TO → Task-Agnostic
- LLMLingua-2-small → USES → Python Dependencies
- torch → PART_OF → LLMLingua-2-small
- transformers → PART_OF → LLMLingua-2-small
- tiktoken → PART_OF → LLMLingua-2-small
- numpy → PART_OF → LLMLingua-2-small
- nltk → PART_OF → LLMLingua-2-small
- accelerate → PART_OF → LLMLingua-2-small
- PyTorch → PART_OF → LLMLingua-2-small
- @atjsh/llmlingua-2 → RELATED_TO → JavaScript/TypeScript
- @atjsh/llmlingua-2 → USES → @huggingface/transformers
- @atjsh/llmlingua-2 → USES → TensorFlow.js
- Hermes → USES → CPU
- LLMLingua-2-small → RELATED_TO → CPU
- GPT-2 Small → RELATED_TO → CPU
- microsoft/llmlingua-2-bert-base-multilingual-cased-meetingbank → USES → mBERT Base (Multilingual)
- microsoft/llmlingua-2-bert-base-multilingual-cased-meetingbank → RELATED_TO → LLMLingua-2-small
- LLMLingua-2-small → USES → Safetensors
- Hermes → RELATED_TO → OpenVINO
- Hermes → RELATED_TO → ONNX
- Prompt Compression in the Wild → RELATED_TO → LLMLingua-2-small
- Hermes → RELATED_TO → BoL (Bag-of-Words/Lines) Checkpoint-Summary
- LLMLingua-2-small → RELATED_TO → Grammatical Decay
- Grammatical Decay → RELATED_TO → Claude
- Data Distillation → USES → GPT-4
- LLMLingua-2-small → USES → LLaMA
- LLMLingua-2-small → USES → GPT-2 Small
- LLMLingua-2-small → USES → microsoft/llmlingua-2-bert-base-multilingual-cased-meetingbank
- LLMLingua-2-small → USES → microsoft/llmlingua-2-xlm-roberta-large-meetingbank
- LLMLingua-2-small → USES → mBERT Base (Multilingual)
- LLMLingua-2-small → AUTHORED_BY → Microsoft Research
- LLMLingua-2-small → RELATED_TO → GPT-4
- LLaMA → RELATED_TO → LLMLingua-2-small
- GPT-2 Small → RELATED_TO → LLMLingua-2-small
- microsoft/llmlingua-2-xlm-roberta-large-meetingbank → RELATED_TO → LLMLingua-2-small
- mBERT Base (Multilingual) → RELATED_TO → LLMLingua-2-small
- Claude → RELATED_TO → Grammatical Decay
- Apple M1 Pro → RELATED_TO → CPU
- LLMLingua-2-small → USES → LangChain
- LLMLingua-2-small → USES → LlamaIndex
- LLMLinguaCompressor → PART_OF → LangChain
- ContextualCompressionRetriever → USES → LLMLinguaCompressor
- LLMLingua-2-small → USES → GPT-4
- LLMLingua-2-small → USES → PyTorch
- LLMLingua-2-small → USES → @huggingface/transformers
- Hermes → USES → LangChain
- Hermes → USES → LLMLingua-2-small
- LLMLinguaCompressor → USES → microsoft/llmlingua-2-bert-base-multilingual-cased-meetingbank
- Hermes → USES → ONNX
- LangChain → RELATED_TO → Retrieval-Augmented Generation (RAG)
- LlamaIndex → RELATED_TO → Retrieval-Augmented Generation (RAG)
