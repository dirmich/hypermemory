# PRD — Hyper Memory

**Document:** `prd.md`  
**Product Name:** Hyper Memory  
**Status:** Draft v1.0  
**Product Type:** Local-first / Self-hostable AI Memory & Knowledge Engine  
**Primary Interfaces:** REST API, MCP, Obsidian Vault, CLI  
**Core Technologies:** Rust, TypeScript/Bun, Vector Search, FTS, Temporal Graph, LLM, Markdown/Obsidian

---

# 1. 제품 개요

## 1.1 한 줄 설명

Hyper Memory는 대화, 문서, 코드, 이메일, 노트 등에서 장기 기억을 자동 추출하고, 시간에 따라 갱신되는 Memory Graph를 구축한 뒤, 이를 사람이 읽고 수정할 수 있는 LLM Wiki와 Obsidian Vault로 자동 변환하는 로컬 우선 AI 메모리 플랫폼이다.

## 1.2 문제 정의

기존 AI 에이전트와 RAG 시스템에는 다음 문제가 있다.

1. 세션이 바뀌면 이전 대화와 맥락을 잊는다.
2. 전체 과거 대화를 매번 LLM Context에 넣으면 비용과 토큰이 폭증한다.
3. 일반적인 RAG는 문서 검색은 잘하지만 "사용자의 현재 상태"나 "이전 사실이 새로운 사실로 바뀌었음"을 잘 처리하지 못한다.
4. Vector DB는 유사도 검색에는 강하지만 시간 관계, 모순, 사실 갱신, 인간이 읽을 수 있는 구조화된 지식 표현이 부족하다.
5. AI가 내부적으로 기억을 저장해도 사용자가 그 내용을 직접 확인하거나 수정하기 어렵다.
6. Obsidian은 사람이 쓰기 좋은 지식 관리 도구이지만 자동 기억 추출, 시맨틱 검색, 시간 기반 사실 관리가 부족하다.
7. 장기간 사용할 경우 벡터와 메모리 객체가 계속 증가해 RAM과 저장 공간이 커진다.
8. 여러 AI 도구를 함께 쓸 경우 각 도구마다 기억이 분리되어 중복 저장과 컨텍스트 단절이 발생한다.
9. 기존 메모리 솔루션은 SaaS 중심이 많아 민감한 데이터의 완전한 로컬 운영이 어렵다.

## 1.3 해결 방향

Hyper Memory는 아래 계층을 결합한다.

```text
Raw Inputs
   ↓
Ingestion
   ↓
Atomic Memories
   ↓
Temporal Memory Graph
   ↓
Entity & Topic Layer
   ↓
LLM Wiki Compiler
   ↓
Markdown / Obsidian Vault
```

검색 시에는:

```text
Query
  ↓
Query Router
  ├─ Profile / KV
  ├─ Exact Lookup
  ├─ FTS / BM25
  ├─ Vector ANN
  ├─ Temporal Graph
  └─ Wiki / Entity Graph
        ↓
      Rerank
        ↓
Compact Context
```

---

# 2. 제품 비전

## 2.1 최종 목표

AI가 사용자의 기억을 단순히 저장하는 것을 넘어 다음을 수행하게 한다.

- 무엇이 중요한지 판단한다.
- 오래된 사실과 새로운 사실을 구분한다.
- 모순되는 기억을 자동 갱신한다.
- 관련 사실을 연결한다.
- 필요 없는 기억을 약화하거나 보관한다.
- 반복되는 기억을 하나로 통합한다.
- 사람이 읽을 수 있는 Wiki 문서로 재구성한다.
- 사용자가 Wiki를 수정하면 그 수정 내용을 Memory Graph에 반영한다.
- 필요한 순간에만 필요한 정보를 매우 적은 토큰으로 AI에게 제공한다.
- 여러 AI Agent가 동일한 장기기억을 공유할 수 있게 한다.

## 2.2 핵심 원칙

1. **Memory와 Document는 분리한다.**
2. **Obsidian은 Source of Truth가 아니라 Human-facing Projection이다.**
3. **모든 입력을 LLM과 Embedding에 보내지 않는다.**
4. **Exact/FTS 검색으로 해결되는 것은 Vector Search를 사용하지 않는다.**
5. **오래된 Memory는 Hot/Warm/Cold 계층으로 이동한다.**
6. **Memory Graph는 시간 관계를 기본 개념으로 가진다.**
7. **사용자가 AI의 기억을 확인·수정·고정·삭제할 수 있어야 한다.**
8. **로컬에서 완전 구동 가능해야 한다.**
9. **저사양 서버에서도 제한된 RAM으로 동작해야 한다.**
10. **LLM Provider와 Embedding Provider를 교체 가능하게 만든다.**
11. **기억은 원본 출처와 항상 연결되어야 한다.**
12. **LLM이 생성한 추론은 명시적 사실과 구분하여 저장한다.**

---

# 3. 주요 사용자

## 3.1 AI 개발자

필요 기능:

- Agent 장기기억
- RAG
- User Profile
- MCP
- REST API
- Project Memory
- Codebase Memory
- 자체 호스팅
- 모델 공급자 교체

## 3.2 개인 사용자

필요 기능:

- 모든 대화의 장기기억
- Obsidian 자동 Wiki
- 과거 대화 검색
- 사람/프로젝트/아이디어 정리
- 자동 Daily Note
- AI가 기억한 내용 확인
- 직접 수정 및 삭제

## 3.3 팀 / 조직

필요 기능:

- 프로젝트별 공유 메모리
- 사용자별 접근권한
- 문서 기반 조직 지식
- 결정사항 추적
- 변경 이력
- Agent 공유 컨텍스트

---

# 4. 핵심 사용 사례

## 4.1 AI 장기기억

사용자가 AI에게 다음과 같이 말한다.

```text
나는 새 프로젝트에서 PostgreSQL을 사용하기로 했어.
```

Hyper Memory는 이를 원문 그대로만 저장하지 않고 다음처럼 처리한다.

```text
Atomic Memory:
사용자는 현재 프로젝트에서 PostgreSQL을 사용하기로 결정했다.

Entity:
PostgreSQL
Current Project

Relationship:
Current Project --USES--> PostgreSQL

Temporal:
valid_from = 현재 시각
```

나중에:

```text
PostgreSQL 대신 ClickHouse로 바꾸기로 했어.
```

라고 하면:

```text
Old Memory:
Project uses PostgreSQL
is_latest = false

New Memory:
Project uses ClickHouse
is_latest = true

Relation:
New Memory UPDATES Old Memory
```

## 4.2 Obsidian Wiki 자동 생성

기억이 누적되면 Hyper Memory는 자동으로 다음 문서를 생성한다.

```text
vault/
├── People/
├── Projects/
├── Technologies/
├── Decisions/
├── Topics/
├── Daily/
└── Sources/
```

예:

```text
Projects/Hyper Memory.md
Technologies/PostgreSQL.md
Technologies/Rust.md
Decisions/Storage Architecture.md
```

## 4.3 Obsidian 수정 → Memory Update

사용자가 Obsidian에서:

```text
현재 DB는 PostgreSQL을 사용한다.
```

를

```text
현재 DB는 ClickHouse로 이전 중이다.
```

로 수정하면 Hyper Memory는 Markdown diff를 감지한다.

그 후:

```text
Wiki edit
  ↓
Diff Engine
  ↓
Semantic Change Detector
  ↓
Memory Update Candidate
  ↓
Auto-apply 또는 사용자 승인
```

## 4.4 프로젝트별 기억

예:

```text
container/project:
hyper-memory
movie-app
github-agent
personal
```

각 프로젝트는 기본적으로 분리한다.

필요 시 Cross-project 검색을 허용한다.

---

# 5. 제품 아키텍처

## 5.1 전체 구조

```text
                    ┌────────────────────────┐
                    │       Clients          │
                    │                        │
                    │ Chat / Agent / CLI     │
                    │ Obsidian / Web UI      │
                    └────────────┬───────────┘
                                 │
                    ┌────────────▼───────────┐
                    │       API Layer        │
                    │ REST / MCP / SDK       │
                    └────────────┬───────────┘
                                 │
                  ┌──────────────▼──────────────┐
                  │       Query Router          │
                  └───────┬──────┬──────┬──────┘
                          │      │      │
                   Profile│      │      │Graph
                          │      │      │
                       FTS│   Vector    │
                          │      │      │
                          └──────┼──────┘
                                 │
                         ┌───────▼────────┐
                         │    Reranker    │
                         └───────┬────────┘
                                 │
                         Compact Context
                                 │
                    ┌────────────▼───────────┐
                    │       LLM / Agent      │
                    └────────────────────────┘
```

Ingestion:

```text
Source
  ↓
Extractor
  ↓
Normalizer
  ↓
Fingerprint / Dedup
  ↓
Importance Filter
  ↓
Chunker
  ↓
Embedding
  ↓
Memory Extractor
  ↓
Temporal Graph
  ↓
Entity Resolver
  ↓
Wiki Compiler
  ↓
Obsidian Vault
```

---

# 6. 기술 스택

## 6.1 Core Engine

권장:

```text
Rust
```

담당:

- Storage
- Vector Index
- FTS
- Temporal Graph
- Cache
- Deduplication
- Memory lifecycle
- Obsidian diff
- Sync engine

추천 라이브러리 후보:

```text
HTTP/API       axum
Async          tokio
FTS            Tantivy
Hash           BLAKE3
Serialization  postcard / rkyv
Cache          moka
Storage        redb / RocksDB / custom mmap storage
Vector         HNSW / DiskANN 계열
```

## 6.2 API / Integration Layer

초기에는 Rust API 단독 구성을 우선한다.

필요 시:

```text
TypeScript/Bun
```

레이어를 추가해 SDK, Web UI, Agent Integration을 담당한다.

## 6.3 Web UI

권장:

```text
React / Next.js 또는 React + Vite
```

주요 화면:

- Memories
- Wiki
- Graph
- Sources
- Search
- Model settings
- Storage usage
- Sync status

---

# 7. 저장 구조

## 7.1 Document

원본 입력.

```text
Document
- id
- container_id
- source_type
- source_uri
- content_hash
- title
- mime_type
- created_at
- updated_at
- imported_at
- metadata
```

## 7.2 Chunk

```text
Chunk
- id
- document_id
- sequence
- content_hash
- text_ref
- embedding_ref
- token_count
- created_at
```

## 7.3 Memory

```text
Memory
- id
- container_id
- canonical_text
- type
- confidence
- importance
- source_id
- source_chunk_id
- created_at
- valid_from
- valid_until
- last_accessed_at
- access_count
- is_latest
- is_static
- is_pinned
- state
```

Memory `state`:

```text
hot
warm
cold
archived
deleted
```

Memory `type`:

```text
fact
preference
episode
decision
goal
task
relationship
inference
temporary
```

## 7.4 Edge

```text
Edge
- source_memory_id
- target_memory_id
- relation
- weight
- created_at
```

relation:

```text
UPDATES
EXTENDS
DERIVES
RELATED
CONTRADICTS
SUPPORTS
MENTIONS
BELONGS_TO
```

내부 표현은 가능한 한 compact integer ID를 사용한다.

## 7.5 Entity

```text
Entity
- id
- canonical_name
- type
- aliases
- description
- created_at
- updated_at
```

Entity type:

```text
person
project
organization
technology
place
concept
document
event
product
custom
```

---

# 8. Temporal Memory Graph

## 8.1 목적

기억이 단순히 축적되는 것이 아니라 "현재 무엇이 사실인가"를 관리한다.

## 8.2 시간 모델

모든 Memory는 필요 시 다음을 가진다.

```text
valid_from
valid_until
observed_at
created_at
```

예:

```text
2026-01:
User lives in Seoul.

2027-03:
User moved to Busan.
```

검색 결과는 기본적으로 최신 사실을 우선한다.

## 8.3 Update Detection

새 기억 생성 시 관련 기억 후보를 찾는다.

```text
new memory
 ↓
entity match
 ↓
FTS candidates
 ↓
vector candidates
 ↓
temporal classifier
```

판단:

```text
NEW
UPDATE
EXTEND
CONTRADICTION
DUPLICATE
```

## 8.4 Derived Memory

명시적 사실과 추론 사실을 구분한다.

```text
source_type = explicit
source_type = inferred
```

Inference에는 반드시:

```text
confidence
source_memory_ids[]
```

를 저장한다.

---

# 9. Memory Consolidation

## 9.1 목적

장기간 사용 시 기억 수가 무한 증가하는 문제를 막는다.

예:

```text
Rust를 좋아한다.
Rust 프로젝트를 하고 있다.
Rust 프로그램을 자주 만든다.
Rust를 선호한다.
```

통합 결과:

```text
사용자는 Rust를 선호하며 여러 프로젝트에서 사용한다.
```

## 9.2 통합 정책

다음 조건을 만족하면 consolidation 후보가 된다.

- 같은 entity
- 높은 semantic similarity
- 일정 횟수 이상 반복
- 내용이 충돌하지 않음
- 시간적으로 유효함

## 9.3 보존 정책

원 기억은 삭제하지 않고 archive 처리한다.

```text
Canonical Memory
   ↓
derived_from:
memory_12
memory_15
memory_18
```

---

# 10. Memory Lifecycle

## 10.1 Hot Memory

기준 예:

```text
최근 30일
최근 접근 빈도 높음
Pinned
중요도 높음
```

저장:

```text
RAM optimized index
```

## 10.2 Warm Memory

기준:

```text
30일~1년
중간 접근 빈도
```

저장:

```text
compressed index
mmap
```

## 10.3 Cold Memory

기준:

```text
1년 이상
접근 거의 없음
```

저장:

```text
disk index
SSD
```

## 10.4 Promotion

Cold/Warm 기억이 자주 검색되면 Hot으로 승격한다.

## 10.5 Decay

Episode와 Temporary Memory는 시간이 지나며 중요도를 낮춘다.

---

# 11. Vector Engine

## 11.1 목표

- 낮은 RAM
- 빠른 ANN
- 대규모 확장
- Embedding Provider 독립

## 11.2 Vector Format

초기 지원:

```text
FP32
FP16
INT8
```

권장 기본:

```text
384d FP16
```

저메모리 모드:

```text
384d INT8
```

## 11.3 검색 전략

```text
INT8 ANN
 ↓
Top 50
 ↓
FP16 / exact rerank
 ↓
Top 10
```

## 11.4 Index Policy

```text
< 1M vectors
HNSW

1M ~ 20M
HNSW + quantization

20M+
DiskANN 계열 또는 SSD-aware ANN
```

---

# 12. Full Text Search

## 12.1 엔진

권장:

```text
Tantivy
```

## 12.2 역할

FTS는 Vector Search의 대체재가 아니라 Query Router의 핵심 경로이다.

예:

```text
"JWT라는 단어가 있는 문서"
```

→ FTS만 수행.

```text
"전에 인증 관련해서 어떤 구조를 얘기했지?"
```

→ FTS + Vector + Graph.

## 12.3 Hybrid Search

```text
BM25 score
semantic score
recency score
importance score
graph score
```

최종:

```text
final_score =
    w1 * lexical
  + w2 * semantic
  + w3 * recency
  + w4 * importance
  + w5 * graph
```

가중치는 query intent에 따라 조정한다.

---

# 13. Query Router

## 13.1 목적

매 요청마다 모든 검색엔진을 실행하지 않는다.

## 13.2 Query Types

```text
PROFILE
EXACT
LEXICAL
SEMANTIC
TEMPORAL
GRAPH
HYBRID
SOURCE_LOOKUP
```

## 13.3 예

```text
"내 이름이 뭐야?"
→ PROFILE

"PostgreSQL이라는 단어가 들어간 기록"
→ FTS

"전에 DB를 바꾸자고 했던 얘기"
→ SEMANTIC + TEMPORAL

"Hyper Memory와 관련된 프로젝트를 보여줘"
→ ENTITY + GRAPH
```

## 13.4 Router 구현

v1:

```text
규칙 기반 + small classifier
```

v2:

```text
Small local LLM
```

---

# 14. LLM Memory Extraction

## 14.1 모든 메시지를 LLM에 보내지 않는다

먼저 Cheap Filter를 사용한다.

```text
Input
 ↓
Rule Filter
 ↓
Small Classifier
 ↓
Memory Candidate?
```

예:

제외:

```text
응
ㅋㅋ
고마워
알겠어
```

후보:

```text
나는 Rust를 주로 사용해.
다음달에 일본에 갈 예정이야.
이 프로젝트 DB는 PostgreSQL로 하자.
```

## 14.2 Extraction Output

```json
{
  "memories": [
    {
      "type": "decision",
      "text": "The project will use PostgreSQL",
      "entities": ["project", "PostgreSQL"],
      "confidence": 0.98,
      "importance": 0.82
    }
  ]
}
```

## 14.3 Large LLM 호출 조건

큰 모델은 다음 경우에만 사용한다.

- 복잡한 관계 추론
- 모순 판단이 어려움
- 여러 문서를 종합해야 함
- Wiki synthesis
- Entity ambiguity

---

# 15. LLM Wiki

## 15.1 역할

Memory Graph를 사람이 읽을 수 있는 지식 문서로 변환한다.

LLM Wiki는 Source of Truth가 아니라 Memory Graph의 Projection이다.

## 15.2 기본 Vault 구조

```text
HyperMemoryVault/
├── Home.md
├── People/
├── Projects/
├── Technologies/
├── Organizations/
├── Topics/
├── Decisions/
├── Goals/
├── Daily/
├── Sources/
└── System/
```

## 15.3 Wiki Page Frontmatter

```yaml
---
hyper_memory_id: project_0012
type: project
aliases:
  - HyperMemory
created: 2026-09-19
updated: 2026-09-19
confidence: 0.98
managed: true
---
```

## 15.4 자동 영역 / 사용자 영역 분리

```markdown
# Hyper Memory

<!-- HYPER_MEMORY:AUTO:BEGIN -->

## Summary

...

## Current State

...

## Related

- [[Rust]]
- [[Obsidian]]
- [[Temporal Memory]]

<!-- HYPER_MEMORY:AUTO:END -->

## My Notes

사용자가 자유롭게 작성하는 영역.
```

Hyper Memory는 AUTO 영역만 갱신한다.

## 15.5 Wikilink

Entity 관계는 가능한 경우:

```text
[[Rust]]
[[PostgreSQL]]
[[Hyper Memory]]
```

형태로 출력한다.

---

# 16. Wiki Compiler

## 16.1 목적

LLM에게 DB를 그대로 넘기지 않는다.

먼저 deterministic intermediate representation을 생성한다.

```json
{
  "entity": {
    "id": "project_001",
    "name": "Hyper Memory",
    "type": "project"
  },
  "facts": [],
  "decisions": [],
  "events": [],
  "related_entities": [],
  "sources": []
}
```

그 후 LLM은 이 구조만 받아 Markdown을 생성한다.

## 16.2 장점

- Hallucination 감소
- 재생성 가능
- diff가 안정적
- LLM Provider 변경 가능
- 테스트 가능

---

# 17. Obsidian 양방향 동기화

## 17.1 방향 1

```text
Memory Graph
→ Wiki Compiler
→ Markdown
→ Obsidian
```

## 17.2 방향 2

```text
Obsidian edit
→ File Watcher
→ Diff Engine
→ Semantic Change Detector
→ Memory Update Candidate
```

## 17.3 자동 반영 정책

다음은 자동 반영 가능:

```text
태그 변경
alias 추가
사용자 note 추가
핀 설정
```

다음은 기본적으로 승인 요청:

```text
사실 수정
기존 memory 삭제
중요도 변경
entity merge
contradiction 발생
```

설정에서:

```text
manual
safe-auto
full-auto
```

모드를 선택할 수 있다.

---

# 18. Obsidian Control Tags

지원 후보:

```text
#hypermemory/pin
#hypermemory/temporary
#hypermemory/private
#hypermemory/ignore
#hypermemory/archive
#hypermemory/no-ai
```

의미:

```text
pin
→ 중요한 Memory로 유지

temporary
→ decay 대상

private
→ 외부 LLM 전송 금지

ignore
→ indexing 제외

archive
→ Cold storage 이동

no-ai
→ LLM extraction 제외
```

---

# 19. Source Tracking

모든 기억은 원본으로 돌아갈 수 있어야 한다.

예:

```text
Memory
"프로젝트 DB는 PostgreSQL"

source:
conversation_293
message_17
```

Wiki:

```markdown
## Sources

- [[Conversation 2026-09-19#message-17]]
```

웹 UI에서는:

```text
View Source
```

버튼을 제공한다.

---

# 20. Deduplication

## 20.1 Content Fingerprint

정규화:

```text
trim
unicode normalization
whitespace normalization
optional lowercase
```

후:

```text
BLAKE3(normalized_content)
```

## 20.2 Chunk Dedup

같은 hash의 chunk는 재저장하지 않는다.

```text
existing chunk
→ reference만 추가
```

## 20.3 Embedding Cache

key:

```text
embedding_model_id
+
content_hash
```

같은 모델과 같은 text이면 embedding을 재사용한다.

---

# 21. Incremental Re-ingestion

파일이 변경되면 전체를 다시 처리하지 않는다.

```text
Old File
 ↓
chunks A B C D

New File
 ↓
chunks A B X D
```

처리:

```text
A reuse
B reuse
X embed/process
D reuse
```

---

# 22. User Profile

## 22.1 구조

```text
Static Profile
Dynamic Profile
Active Projects
Preferences
Recent Decisions
Recent Goals
```

## 22.2 Profile Cache

Profile은 매 요청 생성하지 않는다.

```text
memory changed
 ↓
profile dirty
 ↓
incremental updater
 ↓
cached profile
```

Profile API는 가능한 경우 KV lookup으로 반환한다.

---

# 23. API

## 23.1 Add

```http
POST /v1/documents
```

```json
{
  "content": "...",
  "container": "user_123",
  "custom_id": "conversation_193",
  "source_type": "conversation"
}
```

## 23.2 Search

```http
POST /v1/search
```

```json
{
  "query": "전에 DB 관련해서 무엇을 결정했지?",
  "container": "project_hyper_memory",
  "limit": 10
}
```

## 23.3 Profile

```http
GET /v1/profile/{container}
```

## 23.4 Memories

```http
GET /v1/memories
POST /v1/memories
PATCH /v1/memories/{id}
DELETE /v1/memories/{id}
```

## 23.5 Wiki

```http
POST /v1/wiki/sync
GET /v1/wiki/pages
POST /v1/wiki/rebuild
```

## 23.6 Graph

```http
GET /v1/graph/entity/{id}
GET /v1/graph/memory/{id}
```

---

# 24. MCP Server

필수 MCP Tools:

```text
memory_add
memory_search
memory_get
memory_forget
memory_update

profile_get

wiki_search
wiki_get_page
wiki_update_page

project_context
source_get
```

예:

```text
Agent starts
 ↓
profile_get
 ↓
project_context
 ↓
normal work
```

필요할 때:

```text
memory_search
```

를 호출한다.

---

# 25. Agent Integrations

1차:

```text
Claude Code
Codex
Cursor
OpenCode
Hermes
VS Code
```

2차:

```text
OpenClaw
LangGraph
LangChain
Vercel AI SDK
n8n
```

---

# 26. Local-first Architecture

모든 핵심 기능은 인터넷 없이 실행 가능해야 한다.

```text
Local LLM
Ollama / llama.cpp / vLLM

Local Embedding
BGE / E5 계열

Local Storage

Local Obsidian Vault
```

외부 Provider는 옵션이다.

---

# 27. Privacy

## 27.1 Data Classification

```text
public
internal
private
secret
```

## 27.2 외부 모델 전송 제한

`private` 또는 `secret` memory는 설정에 따라:

```text
local LLM only
```

로 강제할 수 있다.

## 27.3 암호화

지원 목표:

```text
at-rest encryption
TLS
API token
Vault encryption optional
```

---

# 28. Storage Engine

## 28.1 Logical Storage

```text
documents
chunks
memories
entities
edges
vectors
sources
profiles
wiki_state
```

## 28.2 Physical Storage

메타데이터와 대용량 데이터를 분리한다.

```text
metadata DB
text blob
vector files
ANN index
FTS index
graph adjacency
```

## 28.3 mmap

Warm/Cold 데이터는 가능한 경우 mmap을 사용한다.

---

# 29. Compact Graph

Graph를 범용 JSON 객체로 보관하지 않는다.

개념적으로:

```text
u32 source
u32 target
u8 relation
f16 weight
```

구조를 지향한다.

대규모 graph는 CSR 형태 검토:

```text
offset[]
target[]
relation[]
weight[]
```

---

# 30. Memory Budget

서버 설정 예:

```text
HYPER_MEMORY_RAM_LIMIT=2gb
HYPER_MEMORY_INGEST_CONCURRENCY=2
HYPER_MEMORY_VECTOR_CACHE=512mb
HYPER_MEMORY_HOT_MEMORY_LIMIT=250000
```

RAM limit 초과 시 ingest를 queue에 유지한다.

검색은 항상 우선 처리한다.

---

# 31. Background Jobs

```text
memory extraction
consolidation
wiki generation
profile refresh
tier migration
duplicate cleanup
index compaction
cold archive
```

우선순위:

```text
P0 Query
P1 User write
P2 Ingestion
P3 Wiki
P4 Consolidation
P5 Maintenance
```

---

# 32. Event System

내부 Event Bus:

```text
DocumentAdded
ChunkCreated
MemoryCreated
MemoryUpdated
MemoryArchived
EntityCreated
WikiDirty
ProfileDirty
VaultChanged
```

각 모듈이 느슨하게 연결되도록 한다.

---

# 33. Queue

초기:

```text
embedded persistent queue
```

확장형:

```text
Redis / NATS / Kafka optional
```

단일 사용자 로컬 설치는 외부 Queue가 필요 없어야 한다.

---

# 34. Search Pipeline

기본:

```text
1 Query classification
2 Container scope
3 Metadata prefilter
4 Exact / FTS
5 Vector ANN if required
6 Temporal filter
7 Graph expansion
8 Scoring
9 Dedup
10 Compact context generation
```

Graph expansion 기본값:

```text
depth = 1
```

필요 시:

```text
depth = 2
```

---

# 35. Context Compiler

검색 결과를 원문 그대로 LLM에 전달하지 않는다.

예:

```text
20 search results
 ↓
dedup
 ↓
current facts preferred
 ↓
source merge
 ↓
token budget
 ↓
compact context
```

API:

```text
POST /v1/context
```

option:

```text
max_tokens
include_sources
include_profile
include_history
```

---

# 36. Token Budget

기본 목표:

```text
profile        200~500 tokens
memories       300~800
source context 500~1500
```

일반적인 질의에서는 전체 context를:

```text
< 2,000 tokens
```

로 제한하는 것을 목표로 한다.

---

# 37. Wiki Generation Policy

Wiki는 모든 Memory 추가마다 즉시 재생성하지 않는다.

```text
Memory change
 ↓
Wiki page dirty
 ↓
debounce
 ↓
batch compile
```

예:

```text
30초 이내 변경은 묶어서 한 번 생성
```

대규모 import:

```text
import complete
→ batch wiki build
```

---

# 38. Daily Notes

자동 Daily:

```text
Daily/2026-09-19.md
```

포함:

```text
오늘 추가된 기억
대화한 프로젝트
결정사항
새로운 사람/Entity
수정된 사실
진행 중 Task
```

사용자 작성 영역은 유지한다.

---

# 39. Decision Memory

프로젝트 개발에서 특히 중요하다.

예:

```text
Decision:
Core engine will be written in Rust.

Reason:
Memory efficiency and predictable performance.

Alternatives:
TypeScript
Go

Status:
active
```

Wiki:

```text
Decisions/Rust Core Engine.md
```

Agent는 새로운 작업 시작 시 관련 Decision을 자동 조회할 수 있다.

---

# 40. Codebase Memory

Git Repository를 ingest할 경우:

```text
Repository
 ↓
AST-aware chunking
 ↓
Modules
Functions
Types
Architecture
Decisions
```

Wiki 예:

```text
Code/
├── Architecture.md
├── API.md
├── Storage.md
└── Modules/
```

소스가 변경되면 diff 기반으로 관련 memory만 갱신한다.

---

# 41. Knowledge Confidence

각 Memory는 confidence를 갖는다.

```text
1.0 direct user statement
0.9 verified document
0.7 repeated evidence
0.5 inferred
0.3 weak inference
```

최종 검색 ranking에 반영한다.

---

# 42. Contradiction Handling

예:

```text
Memory A:
User prefers Python.

Memory B:
User does not like Python anymore.
```

처리:

```text
B UPDATES A
A.is_latest = false
B.is_latest = true
```

애매하면:

```text
CONTRADICTION unresolved
```

로 보관하고 UI에서 검토 가능하게 한다.

---

# 43. Explicit User Controls

UI에서 지원:

```text
Pin
Forget
Edit
Merge
Split
Archive
Mark wrong
Mark temporary
Set expiration
```

---

# 44. Forgetting

종류:

```text
manual delete
expiration
decay
superseded
low-value pruning
```

기본적으로 삭제보다는 archive를 선호한다.

Hard delete는 사용자 명시 요청에만 수행한다.

---

# 45. Web UI

## Dashboard

표시:

```text
Total memories
Hot/Warm/Cold
Vector count
Disk usage
RAM usage
Wiki pages
Pending ingestion
Pending reviews
```

## Memory Explorer

필터:

```text
entity
type
date
confidence
importance
state
source
```

## Graph View

표시:

```text
Entity nodes
Memory nodes
UPDATES
EXTENDS
DERIVES
RELATED
```

## Wiki Viewer

Obsidian 없이도 브라우저에서 페이지 탐색 가능.

---

# 46. CLI

예:

```bash
hyper-memory start
hyper-memory stop
hyper-memory status

hyper-memory add file.pdf
hyper-memory add ./repo

hyper-memory search "postgres migration"

hyper-memory wiki sync
hyper-memory wiki rebuild

hyper-memory doctor
hyper-memory benchmark
```

---

# 47. Configuration

예:

```toml
[server]
port = 6767

[storage]
data_dir = "./.hyper-memory"

[llm]
provider = "openai-compatible"
base_url = "http://localhost:11434/v1"
model = "qwen"

[embedding]
provider = "local"
model = "bge"
dimensions = 384
format = "int8"

[wiki]
enabled = true
vault_path = "~/Documents/HyperMemoryVault"

[memory]
hot_days = 30
warm_days = 365
```

---

# 48. Plugin Architecture

Provider traits:

```text
LLMProvider
EmbeddingProvider
Extractor
StorageBackend
VectorBackend
SourceConnector
WikiTarget
```

이를 통해 구현 교체를 쉽게 한다.

---

# 49. Connectors

Phase 2 이후:

```text
GitHub
Google Drive
Gmail
Notion
OneDrive
Dropbox
Local Folder
Web
RSS
Slack
```

각 connector는:

```text
source_id
remote_version
last_sync
content_hash
```

를 관리한다.

---

# 50. Performance 목표

로컬 기본 환경 기준 목표.

## Search

```text
Profile lookup
p95 < 10ms

FTS
p95 < 20ms

Hot vector search
p95 < 30ms

Hybrid search
p95 < 80ms
```

LLM reranking 제외 기준.

## Memory

10M vectors 기준 vector raw footprint 목표:

```text
INT8 384d
≈ 3.84GB
```

전체를 RAM에 유지하지 않고 Hot set만 RAM에 적재한다.

## Startup

기본 로컬 서버:

```text
< 3 sec
```

대형 index는 lazy open을 사용한다.

---

# 51. Benchmarks

필수 자체 benchmark:

```text
Memory Recall
Temporal Update
Contradiction
Preference
Multi-session
Entity Resolution
Wiki Accuracy
Search Latency
RAM Usage
Disk Usage
Ingestion Throughput
```

외부 benchmark 고려:

```text
LongMemEval
LoCoMo
ConvoMem
```

---

# 52. 품질 측정

## Retrieval

```text
Recall@5
Recall@10
Recall@20
MRR
NDCG
```

## Memory

```text
Fact precision
Update accuracy
Duplicate rate
False-memory rate
```

## Wiki

```text
Source-grounded accuracy
Broken-link rate
Duplicate page rate
Unwanted rewrite rate
```

---

# 53. Wiki 안전성

LLM이 기존 사용자 내용을 덮어쓰면 안 된다.

필수 규칙:

```text
AUTO 영역만 수정
사용자 영역 보존
Frontmatter의 사용자 필드 보존
충돌 시 새 버전 생성
```

파일 쓰기 전:

```text
temporary file
→ validation
→ atomic rename
```

---

# 54. Versioning

Memory:

```text
revision
```

Wiki:

```text
wiki_revision
```

가능하면 변경 이력을 유지한다.

Obsidian Git 사용 시 자연스럽게 Git History와도 연동 가능하다.

---

# 55. Failure Recovery

각 job은 idempotent해야 한다.

예:

```text
Embedding 중 서버 종료

restart
→ hash 확인
→ 완료된 chunk skip
→ 나머지만 재개
```

---

# 56. Observability

Metrics:

```text
query latency
FTS latency
ANN latency
LLM calls
embedding calls
cache hit
dedup hit
queue length
RAM
disk
wiki compile time
```

Logs는 structured logging 사용.

---

# 57. Security

필수:

```text
API key
scoped token
container authorization
path traversal protection
safe markdown writing
connector credential encryption
```

---

# 58. Multi-tenancy

구조:

```text
tenant
  └── workspace
       └── container
```

예:

```text
personal
  └── coding
       └── hyper-memory
```

검색은 기본적으로 container boundary를 넘지 않는다.

---

# 59. 권한

Role:

```text
owner
admin
editor
viewer
agent
```

Agent token은 필요한 workspace만 접근 가능.

---

# 60. 개발 단계

## Phase 0 — Prototype

목표:

```text
text input
memory extraction
SQLite/redb
basic vector search
basic temporal updates
REST API
```

## Phase 1 — Core

추가:

```text
Rust engine
Tantivy
HNSW
compact graph
dedup
embedding cache
profile
MCP
```

## Phase 2 — Wiki

추가:

```text
Entity layer
Wiki compiler
Obsidian vault generation
file watcher
two-way sync
Daily Notes
```

## Phase 3 — Optimization

추가:

```text
FP16 / INT8
Hot/Warm/Cold
mmap
memory consolidation
query router
small-model filter
```

## Phase 4 — Integrations

```text
GitHub
Drive
Gmail
Notion
Local folders
Agent plugins
```

## Phase 5 — Scale

```text
DiskANN
distributed storage option
remote worker
multi-node ingestion
```

---

# 61. MVP 범위

MVP에 반드시 포함:

```text
Rust server
REST API
MCP
Text ingestion
Markdown ingestion
Memory extraction
Entity extraction
Temporal updates
FTS
Vector search
Profile
Obsidian Wiki export
Wiki AUTO/User section separation
Basic Obsidian change detection
Embedding cache
Content dedup
```

MVP에서 제외 가능:

```text
Video
OCR
Gmail
Drive
Distributed cluster
Multi-region
Advanced RBAC
DiskANN
```

---

# 62. MVP 성공 기준

1. 100,000개의 Memory를 단일 로컬 서버에서 안정적으로 처리.
2. 일반 검색 p95 100ms 이하(LLM 호출 제외).
3. 동일 문서 재입력 시 중복 Embedding 95% 이상 방지.
4. Memory update 관계 정확도 90% 이상 목표.
5. Obsidian Wiki의 사용자 작성 영역 손실 0건.
6. 서버 재시작 후 모든 Index와 Wiki 상태 복구.
7. Claude/Codex/Hermes 중 최소 2개 이상의 Agent와 MCP 연동 성공.
8. Local LLM + Local Embedding만으로 end-to-end 동작.

---

# 63. Non-goals

초기 버전에서는 다음을 목표로 하지 않는다.

- 범용 Graph DB 대체
- 범용 Vector DB 대체
- Notion 전체 대체
- Obsidian 자체 대체
- 완전 자동 지식 생성에 대한 무조건적 신뢰
- 모든 문장을 영구기억으로 저장

---

# 64. 주요 리스크

## LLM Hallucination

대응:

```text
source-bound memory
confidence
explicit/inferred 분리
Wiki IR
```

## Graph Explosion

대응:

```text
depth limit
edge threshold
memory consolidation
```

## Vector Explosion

대응:

```text
dedup
quantization
tiered storage
consolidation
```

## Wiki Rewrite Conflict

대응:

```text
managed block
diff
atomic write
versioning
```

## Local Resource Usage

대응:

```text
RAM budget
worker limit
lazy loading
queue
```

---

# 65. 제품 차별화

Hyper Memory의 핵심 차별점은 단순 Vector Memory가 아니다.

```text
Persistent AI Memory
+
Temporal Graph
+
RAG
+
Human-editable LLM Wiki
+
Obsidian two-way sync
+
Local-first
+
Low-memory Rust Engine
```

특히:

```text
AI Memory ↔ Human Knowledge Base
```

의 양방향 구조가 핵심이다.

---

# 66. 장기 방향

Hyper Memory는 향후 개인 또는 조직의 "AI 기억 운영체제"를 목표로 한다.

```text
Claude
Codex
Hermes
Cursor
Custom Agents
Local LLM
```

모두가 하나의 Hyper Memory를 공유한다.

각 Agent는 전체 과거 데이터를 직접 읽지 않고 Hyper Memory에게 필요한 Context만 요청한다.

```text
Agent
 ↓
Hyper Memory
 ↓
Profile + Current Facts + Relevant History + Sources
 ↓
Compact Context
```

동시에 사람은 같은 지식을:

```text
Obsidian
Web Wiki
Graph UI
```

에서 직접 읽고 수정할 수 있다.

---

# 67. 최종 아키텍처 요약

```text
                          Hyper Memory

      ┌────────────────────────────────────────────┐
      │                  Sources                   │
      │ Chat · Files · Code · GitHub · Notes      │
      └───────────────────┬────────────────────────┘
                          │
                    Ingestion Engine
                          │
           Normalize · Hash · Diff · Dedup
                          │
               ┌──────────▼──────────┐
               │   Memory Extractor  │
               └──────────┬──────────┘
                          │
             ┌────────────▼────────────┐
             │ Temporal Memory Graph   │
             └────────────┬────────────┘
                          │
                 Entity Resolution
                          │
     ┌────────────────────┼─────────────────────┐
     │                    │                     │
    FTS                Vector                 Graph
  Tantivy          HNSW/DiskANN          Compact CSR
     │                    │                     │
     └────────────────────┼─────────────────────┘
                          │
                     Query Router
                          │
                       Rerank
                          │
                   Context Compiler
                          │
         ┌────────────────┴────────────────┐
         │                                 │
       Agents                         Wiki Compiler
         │                                 │
 Claude/Codex/etc.                   Markdown
                                           │
                                       Obsidian
                                           │
                                      Human Edit
                                           │
                                      Diff Engine
                                           │
                                  Memory Update Candidate
```

---

# 68. 제품 정의

Hyper Memory는 단순한 AI Memory DB가 아니다.

정확한 제품 정의는 다음과 같다.

> **Hyper Memory is a local-first temporal memory and knowledge engine that turns conversations, documents, and agent activity into persistent machine memory and a human-editable living wiki.**

핵심 목표는 AI와 사람이 같은 지식 기반을 서로 다른 형태로 공유하게 만드는 것이다.

```text
AI
→ structured memory graph

Human
→ Obsidian / Wiki

Same Knowledge
```

이를 통해 AI의 장기기억은 보이지 않는 블랙박스가 아니라 사람이 확인하고 수정할 수 있는 살아 있는 지식 시스템이 된다.
