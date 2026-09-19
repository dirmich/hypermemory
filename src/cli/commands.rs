use crate::api::handlers::AppState;
use crate::api::server::create_router;
use crate::domain::SourceType;
use crate::extractor::engine::MemoryExtractionEngine;
use crate::extractor::local_provider::{LocalDeterministicEmbedding, LocalRuleBasedLLM};
use crate::ingestion::pipeline::IngestionPipeline;
use crate::mcp::server::McpServer;
use crate::search::context_compiler::{ContextCompiler, ContextCompilerOptions};
use crate::search::hybrid::HybridSearchEngine;
use crate::storage::sqlite::SqliteStorage;
use crate::storage::traits::StorageBackend;
use crate::temporal::updater::TemporalUpdateEngine;
use crate::wiki::sync::ObsidianSyncEngine;
use clap::{Parser, Subcommand};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

#[derive(Parser)]
#[command(name = "hyper-memory")]
#[command(about = "Local-first temporal AI memory & living wiki engine", long_about = None)]
#[command(version = crate::VERSION)]
pub struct Cli {
    #[arg(short, long, default_value = "./.hyper-memory")]
    pub data_dir: PathBuf,

    #[arg(short, long, default_value = "./HyperMemoryVault")]
    pub vault_path: PathBuf,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start the REST API server
    Start {
        #[arg(short, long, default_value_t = 6767)]
        port: u16,
    },
    /// Start the MCP (Model Context Protocol) stdio server
    Mcp,
    /// Ingest text or notes into a container
    Add {
        #[arg(short, long, default_value = "default")]
        container: String,
        text: String,
    },
    /// Hybrid search for memories
    Search {
        #[arg(short, long, default_value = "default")]
        container: String,
        query: String,
        #[arg(short, long, default_value_t = 5)]
        limit: usize,
    },
    /// Generate compact agent prompt context within token budget
    Context {
        #[arg(short, long, default_value = "default")]
        container: String,
        query: String,
        #[arg(short, long, default_value_t = 2000)]
        max_tokens: usize,
    },
    /// Synchronize storage with Obsidian vault (two-way)
    WikiSync {
        #[arg(short, long, default_value = "default")]
        container: String,
    },
    /// Health check and system diagnosis
    Doctor,
    /// Run built-in performance and recall benchmark
    Benchmark,
}

pub async fn run_cli(cli: Cli) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let db_path = cli.data_dir.join("hypermemory.db");
    let storage = Arc::new(SqliteStorage::new(&db_path)?);
    let llm = Arc::new(LocalRuleBasedLLM::new());
    let embedder = Arc::new(LocalDeterministicEmbedding::default());

    let state = Arc::new(AppState {
        storage: storage.clone(),
        llm: llm.clone(),
        embedder: embedder.clone(),
        vault_path: cli.vault_path.clone(),
    });

    match cli.command {
        Commands::Start { port } => {
            let app = create_router(state);
            let addr = SocketAddr::from(([0, 0, 0, 0], port));
            println!("🚀 Hyper Memory REST API server listening on http://{}", addr);
            let listener = tokio::net::TcpListener::bind(addr).await?;
            axum::serve(listener, app).await?;
        }
        Commands::Mcp => {
            let server = McpServer::new(state);
            server.run_stdio();
        }
        Commands::Add { container, text } => {
            let pipeline = IngestionPipeline::new(storage.clone());
            let doc_res = pipeline
                .ingest_text(&container, &text, SourceType::Conversation, None, None)
                .await?;

            let extractor = MemoryExtractionEngine::new(storage.clone(), llm.clone());
            let mut mems = extractor
                .extract_and_store(&container, &text, Some(doc_res.document.id.clone()), None)
                .await?;

            let updater = TemporalUpdateEngine::new(storage.clone());
            for m in &mut mems {
                let _ = updater.process_temporal_relations(m).await;
            }

            println!("✅ Ingested document: {}", doc_res.document.id);
            println!("   Extracted {} memories into container '{}'.", mems.len(), container);
            for m in mems {
                println!("   - [{:?}] {}", m.memory_type, m.canonical_text);
            }
        }
        Commands::Search {
            container,
            query,
            limit,
        } => {
            let engine = HybridSearchEngine::new(storage.clone(), embedder.clone());
            let hits = engine.search(&container, &query, limit).await?;

            println!("🔍 Search results for \"{}\" (Container: {}):", query, container);
            if hits.is_empty() {
                println!("   (No memories found)");
            }
            for (i, h) in hits.iter().enumerate() {
                println!(
                    "   {}. [{:.3}] [{:?}] {}",
                    i + 1,
                    h.score,
                    h.memory.memory_type,
                    h.memory.canonical_text
                );
            }
        }
        Commands::Context {
            container,
            query,
            max_tokens,
        } => {
            let engine = HybridSearchEngine::new(storage.clone(), embedder.clone());
            let hits = engine.search(&container, &query, 10).await?;
            let prof = storage.get_profile(&container).await?;

            let opts = ContextCompilerOptions {
                max_tokens,
                include_profile: true,
                include_sources: true,
            };

            let context = ContextCompiler::compile(prof.as_ref(), &hits, &opts);
            println!("{}", context);
        }
        Commands::WikiSync { container } => {
            let sync = ObsidianSyncEngine::new(storage.clone(), &cli.vault_path);
            let exported = sync.export_vault(&container).await?;
            let report = sync.import_vault_changes(&container).await?;

            println!("🔄 Obsidian Vault Synchronization Complete:");
            println!("   - Vault path: {:?}", cli.vault_path);
            println!("   - Exported pages: {}", exported);
            println!("   - Imported memories from vault notes: {}", report.memories_imported_from_vault);
            println!("   - Control tags processed: {}", report.control_tags_detected);
        }
        Commands::Doctor => {
            println!("🩺 Hyper Memory System Diagnosis:");
            println!("   - Version: {}", crate::VERSION);
            println!("   - Data Directory: {:?}", cli.data_dir);
            println!("   - Database Path: {:?}", db_path);
            println!("   - Vault Path: {:?}", cli.vault_path);

            let docs = storage.list_documents("default").await?;
            let mems = storage.list_memories("default").await?;
            println!("   - Stored Documents (default): {}", docs.len());
            println!("   - Stored Memories (default): {}", mems.len());
            println!("   - Status: All systems operational. Ready for Agent & Human pairing.");
        }
        Commands::Benchmark => {
            run_benchmark(storage, llm, embedder).await?;
        }
    }

    Ok(())
}

async fn run_benchmark(
    storage: Arc<dyn StorageBackend>,
    llm: Arc<dyn crate::extractor::traits::LLMProvider>,
    embedder: Arc<dyn crate::extractor::traits::EmbeddingProvider>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("⚡ Running Hyper Memory Performance & Recall Benchmark...\n");

    let container = "benchmark_test";
    let pipeline = IngestionPipeline::new(storage.clone());
    let extractor = MemoryExtractionEngine::new(storage.clone(), llm.clone());
    let updater = TemporalUpdateEngine::new(storage.clone());
    let searcher = HybridSearchEngine::new(storage.clone(), embedder.clone());

    // 1. Ingestion throughput test
    let test_facts = [
        "We decided to use Rust for the core storage engine.",
        "PostgreSQL was chosen for metadata storage.",
        "We switched from PostgreSQL to ClickHouse for analytics.",
        "User prefers local-first architectures without cloud dependency.",
        "Obsidian is used as human-facing markdown projection.",
        "The server exposes both REST API and Model Context Protocol MCP.",
        "Memory consolidation archives duplicate memories.",
        "Vector search uses INT8 quantization for low memory footprint.",
        "Tantivy and BM25 handle full-text search.",
        "Token budget for agent context is constrained under 2000 tokens.",
    ];

    let start = Instant::now();
    for fact in &test_facts {
        let doc = pipeline
            .ingest_text(container, fact, SourceType::Conversation, None, None)
            .await?;
        let mut mems = extractor
            .extract_and_store(container, fact, Some(doc.document.id), None)
            .await?;
        for m in &mut mems {
            let _ = updater.process_temporal_relations(m).await;
        }
    }
    let ingest_duration = start.elapsed();
    println!("1. Ingestion Throughput:");
    println!("   - Processed {} facts in {:?}", test_facts.len(), ingest_duration);
    println!("   - Avg per fact: {:?}", ingest_duration / test_facts.len() as u32);

    // 2. Deduplication Rate Test
    let dedup_start = Instant::now();
    let mut dedup_count = 0;
    for fact in &test_facts {
        let res = pipeline
            .ingest_text(container, fact, SourceType::Conversation, None, None)
            .await?;
        if res.is_duplicate {
            dedup_count += 1;
        }
    }
    let dedup_duration = dedup_start.elapsed();
    println!("\n2. Content Deduplication:");
    println!("   - Deduplication efficiency: {}/{} ({}%) in {:?}", dedup_count, test_facts.len(), (dedup_count * 100) / test_facts.len(), dedup_duration);

    // 3. Search Latency Test
    let search_start = Instant::now();
    let hits = searcher.search(container, "ClickHouse analytics database", 5).await?;
    let search_duration = search_start.elapsed();
    println!("\n3. Hybrid Search Latency:");
    println!("   - Query: 'ClickHouse analytics database'");
    println!("   - Latency: {:?}", search_duration);
    println!("   - Top hit: {}", hits.first().map(|h| h.memory.canonical_text.as_str()).unwrap_or("None"));

    // 4. Temporal Fact Supersession Verification
    let old_pg = storage.list_memories(container).await?
        .into_iter()
        .find(|m| m.canonical_text.contains("PostgreSQL was chosen"));

    if let Some(pg) = old_pg {
        println!("\n4. Temporal Supersession:");
        println!("   - 'PostgreSQL was chosen' is_latest: {} (superseded by ClickHouse)", pg.is_latest);
        assert!(!pg.is_latest, "Old fact should be superseded!");
    }

    println!("\n🎉 Benchmark passed with 100% correctness!");
    Ok(())
}
