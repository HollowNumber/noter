//! Sample content templates for development data generation
//!
//! This module provides template content generators for creating realistic
//! course materials including lectures, assignments, and study materials.

#[cfg(feature = "dev-tools")]
use super::generator::Course;
#[cfg(feature = "dev-tools")]
use fake::{
    Fake,
    faker::lorem::en::{Sentence, Word, Words},
    faker::name::en::Name,
};
#[cfg(feature = "dev-tools")]
use rand::{Rng, SeedableRng, prelude::IndexedRandom, rngs::StdRng};

/// Template generator for course information files
/// Template generator for lecture notes
#[cfg(feature = "dev-tools")]
pub struct LectureTemplate;

#[cfg(feature = "dev-tools")]
impl LectureTemplate {
    pub fn generate(lecture_num: usize, topic: &str, course: &Course, date: &str) -> String {
        let mut seed = [0u8; 32];
        // Fill with some random-ish data
        for (i, byte) in seed.iter_mut().enumerate() {
            *byte = (i as u8).wrapping_mul(17).wrapping_add(42);
        }
        let mut rng = StdRng::from_seed(seed);

        // Generate dynamic content
        let overview = Self::generate_overview(topic, &mut rng);
        let concepts = Self::generate_concepts(&mut rng);
        let examples = Self::generate_examples(course, topic, &mut rng);
        let problems = Self::generate_problems(topic, &mut rng);
        let references = Self::generate_references(lecture_num, &mut rng);

        format!(
            r#"= Lecture {}: {}
*Course*: {} - {} \
*Date*: {} \
*Topic*: {}

== Overview
{}

== Key Concepts

=== Main Topic: {}
{}

=== Secondary Topics
{}

== Examples

{}

== Practice Problems
{}

== Summary
- {}
- {}
- {}

== References
{}

#pagebreak()
"#,
            lecture_num,
            topic,
            course.code,
            course.name,
            date,
            topic,
            overview,
            topic,
            concepts.main_concepts,
            concepts.secondary_topics,
            examples,
            problems,
            Sentence(3..8).fake::<String>(),
            Sentence(3..8).fake::<String>(),
            Sentence(3..8).fake::<String>(),
            references
        )
    }

    fn generate_overview(topic: &str, rng: &mut impl Rng) -> String {
        let approaches = [
            "theoretical foundations and practical applications",
            "real-world use cases and implementation strategies",
            "algorithmic approaches and optimization techniques",
            "design patterns and best practices",
            "problem-solving methodologies and analysis",
        ];

        let contexts = [
            "modern software development",
            "data-driven decision making",
            "scalable system design",
            "performance optimization",
            "maintainable code architecture",
        ];

        format!(
            "Today's lecture covers {} with focus on {}. We will explore how these concepts apply to {} and examine various {} that demonstrate the practical utility of this material.",
            topic,
            approaches[rng.random_range(0..approaches.len())],
            contexts[rng.random_range(0..contexts.len())],
            ["case studies", "examples", "implementations", "scenarios"][rng.random_range(0..4)]
        )
    }

    fn generate_concepts(rng: &mut impl Rng) -> ConceptContent {
        let concept_types = [
            (
                "definition and mathematical foundations",
                "formal specification and properties",
            ),
            (
                "implementation strategies",
                "algorithmic complexity and trade-offs",
            ),
            (
                "design principles",
                "architectural considerations and patterns",
            ),
            (
                "optimization techniques",
                "performance analysis and bottlenecks",
            ),
            (
                "integration approaches",
                "compatibility and interoperability",
            ),
        ];

        let secondary_topics = [
            "Historical development and evolution",
            "Relationship to existing methodologies",
            "Performance characteristics and benchmarks",
            "Common pitfalls and debugging strategies",
            "Industry applications and case studies",
            "Future developments and research directions",
        ];

        let selected_concept = &concept_types[rng.random_range(0..concept_types.len())];
        let selected_secondary: Vec<_> =
            secondary_topics.choose_multiple(rng, 3).cloned().collect();

        ConceptContent {
            main_concepts: format!(
                "- {}: detailed explanation with examples\n- {}: relationship to previous material  \n- Applications and use cases: practical implementations in {}",
                selected_concept.0,
                selected_concept.1,
                [
                    "web development",
                    "data analysis",
                    "system design",
                    "machine learning",
                    "distributed systems"
                ][rng.random_range(0..5)]
            ),
            secondary_topics: selected_secondary
                .iter()
                .map(|topic| format!("- {topic}"))
                .collect::<Vec<_>>()
                .join("\n"),
        }
    }

    fn generate_examples(_course: &Course, topic: &str, rng: &mut impl Rng) -> String {
        let languages = ["python", "rust", "java", "javascript", "cpp"];
        let lang = languages[rng.random_range(0..languages.len())];

        let function_names = [
            "process_data",
            "calculate_result",
            "optimize_algorithm",
            "validate_input",
            "transform_structure",
            "analyze_pattern",
            "generate_output",
            "handle_request",
        ];

        let func_name = function_names[rng.random_range(0..function_names.len())];

        let comments = [
            "Implementation with error handling",
            "Optimized version for large datasets",
            "Demonstration of key principles",
            "Example showing best practices",
            "Simplified version for understanding",
        ];

        format!(
            r#"```{}
# Example implementation for {}
def {}():
    """
    {}

    Returns:
        Processed result demonstrating {} concepts
    """
    # {}
    {}
    return result

# Additional helper functions
def validate_constraints():
    # Ensure input meets requirements
    pass

def performance_benchmark():
    # Measure and optimize execution time
    pass
```"#,
            lang,
            topic,
            func_name,
            Sentence(8..15).fake::<String>(),
            topic,
            comments[rng.random_range(0..comments.len())],
            Self::generate_implementation_steps(rng),
        )
    }

    fn generate_implementation_steps(rng: &mut impl Rng) -> String {
        let steps = [
            "    # Step 1: Initialize data structures",
            "    # Step 2: Process input parameters",
            "    # Step 3: Apply transformation logic",
            "    # Step 4: Validate intermediate results",
            "    # Step 5: Optimize for performance",
            "    # Step 6: Handle edge cases",
        ];

        let count = rng.random_range(3..5);
        steps
            .choose_multiple(rng, count)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn generate_problems(topic: &str, rng: &mut impl Rng) -> String {
        let problem_types = [
            (
                "Theoretical Analysis",
                "Analyze the computational complexity and prove correctness",
            ),
            (
                "Implementation Challenge",
                "Design and implement a solution using the concepts discussed",
            ),
            (
                "Optimization Problem",
                "Improve the performance of the given algorithm or data structure",
            ),
            (
                "Design Question",
                "Create a system architecture that incorporates these principles",
            ),
            (
                "Comparative Study",
                "Compare different approaches and justify your choice",
            ),
        ];

        let selected_problems: Vec<_> = problem_types.choose_multiple(rng, 3).cloned().collect();

        selected_problems
            .iter()
            .enumerate()
            .map(|(i, (title, description))| {
                format!(
                    "{}. **{}**: {} related to {}",
                    i + 1,
                    title,
                    description,
                    topic
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn generate_references(lecture_num: usize, rng: &mut impl Rng) -> String {
        let textbooks = [
            "Introduction to Algorithms (CLRS)",
            "Design Patterns: Elements of Reusable Object-Oriented Software",
            "Clean Code: A Handbook of Agile Software Craftsmanship",
            "System Design Interview",
            "The Art of Computer Programming",
        ];

        let online_resources = [
            "IEEE Computer Society Digital Library",
            "ACM Digital Library research papers",
            "MIT OpenCourseWare lectures",
            "Stanford CS course materials",
            "Google Developer Documentation",
        ];

        let author_email = format!("{}@dtu.dk", Word().fake::<String>().to_lowercase());

        format!(
            "- Course textbook, Chapter {}\n- {}\n- {}\n- Supplementary reading: {}",
            lecture_num % 15 + 1,
            textbooks[rng.random_range(0..textbooks.len())],
            online_resources[rng.random_range(0..online_resources.len())],
            author_email
        )
    }
}

#[cfg(feature = "dev-tools")]
struct ConceptContent {
    main_concepts: String,
    secondary_topics: String,
}

/// Template generator for assignments
#[cfg(feature = "dev-tools")]
pub struct AssignmentTemplate;

#[cfg(feature = "dev-tools")]
impl AssignmentTemplate {
    pub fn generate(
        assignment_num: usize,
        assignment_type: &str,
        course: &Course,
        due_date: &str,
        points: i32,
    ) -> String {
        let mut seed = [0u8; 32];
        // Fill with some random-ish data
        for (i, byte) in seed.iter_mut().enumerate() {
            *byte = (i as u8).wrapping_mul(17).wrapping_add(42);
        }
        let mut rng = StdRng::from_seed(seed);

        let problems = Self::generate_assignment_problems(assignment_type, &mut rng);
        let context = Self::generate_assignment_context(course, &mut rng);
        let resources = Self::generate_resources(&mut rng);

        format!(
            r#"= Assignment {}: {} Assignment
*Course*: {} - {} \
*Due Date*: {} \
*Points*: {} points \
*Type*: {}

== Context
{}

== Instructions
Complete the following tasks related to the course material. Show all work and provide clear explanations for your solutions. {}

{}

== Submission Guidelines
- Submit all files in a single archive ({})
- Include a comprehensive README with setup instructions
- Ensure all code compiles and runs without errors
- Follow the coding standards and style guide discussed in class
- Include unit tests demonstrating correctness

== Grading Criteria
- **Correctness (60%)**: Does the solution work as specified?
- **Code Quality (20%)**: Is the code well-structured, readable, and maintainable?
- **Documentation (15%)**: Are explanations clear and comprehensive?
- **Innovation (5%)**: Does the solution show creativity or optimization?

== Resources
{}

== Collaboration Policy
{}

*Note*: Late submissions will be penalized according to the course policy (-10% per day, maximum 3 days).
"#,
            assignment_num,
            assignment_type,
            course.code,
            course.name,
            due_date,
            points,
            assignment_type,
            context,
            Self::generate_special_instructions(assignment_type, &mut rng),
            problems,
            Self::generate_file_format(&mut rng),
            resources,
            Self::generate_collaboration_policy(&mut rng)
        )
    }

    fn generate_assignment_context(course: &Course, rng: &mut impl Rng) -> String {
        let scenarios = [
            "You are working as a software engineer at a tech startup",
            "Your team has been hired to solve a critical business problem",
            "A client has requested an analysis of their existing system",
            "You are contributing to an open-source project",
            "A research lab needs your expertise to process their data",
        ];

        let challenges = [
            "scalability issues with their current architecture",
            "performance bottlenecks in data processing pipelines",
            "security vulnerabilities that need to be addressed",
            "user experience problems affecting customer satisfaction",
            "integration challenges between legacy and modern systems",
        ];

        format!(
            "{}. They are facing {} and need your expertise in {} to develop a robust solution.",
            scenarios[rng.random_range(0..scenarios.len())],
            challenges[rng.random_range(0..challenges.len())],
            course.name.to_lowercase()
        )
    }

    fn generate_assignment_problems(assignment_type: &str, rng: &mut impl Rng) -> String {
        match assignment_type {
            "Programming" => Self::generate_programming_problems(rng),
            "Theoretical" => Self::generate_theoretical_problems(rng),
            "Analysis" => Self::generate_analysis_problems(rng),
            "Design" => Self::generate_design_problems(rng),
            "Research" => Self::generate_research_problems(rng),
            _ => Self::generate_generic_problems(rng),
        }
    }

    fn generate_programming_problems(rng: &mut impl Rng) -> String {
        let algorithms = [
            "sorting algorithm",
            "search algorithm",
            "graph traversal",
            "dynamic programming solution",
            "data structure",
            "optimization algorithm",
            "parsing system",
            "caching mechanism",
        ];

        let datasets = [
            "real-world financial data",
            "social network connections",
            "sensor readings",
            "text documents",
            "image metadata",
            "log files",
            "user interactions",
        ];

        let algorithm = algorithms[rng.random_range(0..algorithms.len())];
        let dataset = datasets[rng.random_range(0..datasets.len())];

        format!(
            r#"== Problem 1: Core Implementation (35 points)
Implement a {} that efficiently processes {}. Your solution should handle edge cases and demonstrate optimal time complexity.

=== Requirements:
- Clean, well-commented code following best practices
- Comprehensive error handling and input validation
- Unit tests covering normal and edge cases
- Performance analysis with Big-O notation

== Problem 2: Optimization Challenge (30 points)
Given the baseline implementation, optimize it for:
- **Memory efficiency**: Reduce space complexity where possible
- **Runtime performance**: Minimize execution time for large inputs
- **Scalability**: Design for concurrent or distributed processing

```
// Your optimized solution here
// Include benchmarking code to demonstrate improvements
```

== Problem 3: Integration & Testing (25 points)
Create a complete system that integrates your solution with:
- Input/output handling from multiple data sources
- Configuration management for different environments
- Comprehensive logging and monitoring
- Error recovery and graceful degradation

== Problem 4: Documentation & Analysis (10 points)
Provide detailed documentation including:
- API documentation with usage examples
- Performance benchmarks and analysis
- Deployment instructions and dependencies
- Discussion of design decisions and trade-offs"#,
            algorithm, dataset
        )
    }

    fn generate_theoretical_problems(rng: &mut impl Rng) -> String {
        let concepts = [
            "computational complexity theory",
            "formal verification methods",
            "mathematical proofs",
            "algorithm correctness",
            "system modeling",
            "security analysis",
        ];

        format!(
            r#"== Problem 1: Theoretical Foundation (40 points)
Provide a comprehensive analysis of {} including:
- Formal definitions and mathematical foundations
- Proof of key properties and theorems
- Comparison with alternative approaches
- Discussion of limitations and assumptions

== Problem 2: Formal Analysis (35 points)
Construct formal proofs for the following propositions:
- {}
- {}
- {}

Include detailed step-by-step reasoning and cite relevant theorems.

== Problem 3: Critical Evaluation (25 points)
Critically evaluate the theoretical framework by:
- Identifying potential weaknesses or gaps
- Proposing improvements or extensions
- Discussing real-world applicability
- Comparing with competing theories"#,
            concepts[rng.random_range(0..concepts.len())],
            Sentence(8..12).fake::<String>(),
            Sentence(8..12).fake::<String>(),
            Sentence(8..12).fake::<String>()
        )
    }

    fn generate_analysis_problems(rng: &mut impl Rng) -> String {
        let systems = [
            "distributed database system",
            "microservices architecture",
            "machine learning pipeline",
            "real-time processing system",
            "web application framework",
            "mobile app backend",
        ];

        format!(
            r#"== Problem 1: System Analysis (40 points)
Analyze the provided {} by examining:
- Architecture and design patterns used
- Performance characteristics and bottlenecks
- Security considerations and vulnerabilities
- Scalability limitations and solutions

== Problem 2: Comparative Study (35 points)
Compare three different approaches to solving the same problem:
- Document the pros and cons of each approach
- Provide quantitative analysis where possible
- Consider factors like maintainability, cost, and performance
- Recommend the best solution with justification

== Problem 3: Impact Assessment (25 points)
Evaluate the broader implications of implementing your recommended solution:
- Technical debt and maintenance considerations
- Team skill requirements and training needs
- Business impact and ROI analysis
- Risk assessment and mitigation strategies"#,
            systems[rng.random_range(0..systems.len())]
        )
    }

    fn generate_design_problems(rng: &mut impl Rng) -> String {
        let projects = [
            "social media platform",
            "e-commerce system",
            "educational platform",
            "healthcare management system",
            "financial trading platform",
            "IoT monitoring system",
        ];

        format!(
            r#"== Problem 1: System Design (40 points)
Design a scalable {} that supports:
- High availability and fault tolerance
- Real-time data processing and analytics
- Multi-tenant architecture with isolation
- Integration with external services and APIs

Include detailed architecture diagrams and component specifications.

== Problem 2: Database Design (30 points)
Design the data model and database schema:
- Entity-relationship diagrams
- Normalization and optimization strategies
- Indexing and query performance considerations
- Data migration and backup strategies

== Problem 3: API Design (30 points)
Create a comprehensive API specification:
- RESTful endpoints with proper HTTP methods
- Authentication and authorization mechanisms
- Rate limiting and throttling strategies
- Comprehensive documentation with examples"#,
            projects[rng.random_range(0..projects.len())]
        )
    }

    fn generate_research_problems(rng: &mut impl Rng) -> String {
        let topics = [
            "emerging technologies in software engineering",
            "AI ethics and fairness",
            "quantum computing applications",
            "blockchain and distributed ledgers",
            "cybersecurity and privacy",
            "sustainable computing practices",
        ];

        format!(
            r#"== Problem 1: Literature Review (40 points)
Conduct a comprehensive literature review on {}:
- Survey recent research papers (last 3-5 years)
- Identify key trends, debates, and open questions
- Synthesize findings from multiple sources
- Propose areas for future research

== Problem 2: Experimental Design (35 points)
Design and conduct an experiment to investigate:
- Clear hypothesis and research questions
- Methodology and experimental setup
- Data collection and analysis plan
- Statistical methods for validation

== Problem 3: Innovation Proposal (25 points)
Propose a novel solution or improvement:
- Identify gaps in current approaches
- Present your innovative idea with justification
- Discuss implementation challenges and solutions
- Evaluate potential impact and adoption barriers"#,
            topics[rng.random_range(0..topics.len())]
        )
    }

    fn generate_generic_problems(_rng: &mut impl Rng) -> String {
        format!(
            r#"== Problem 1: Core Concepts (35 points)
Demonstrate your understanding of the fundamental concepts by:
- {}
- {}
- {}

== Problem 2: Practical Application (40 points)
Apply the theoretical knowledge to solve a real-world problem:
- {}
- {}

== Problem 3: Critical Thinking (25 points)
Analyze and evaluate different approaches:
- {}
- {}"#,
            Sentence(8..15).fake::<String>(),
            Sentence(8..15).fake::<String>(),
            Sentence(8..15).fake::<String>(),
            Sentence(10..18).fake::<String>(),
            Sentence(10..18).fake::<String>(),
            Sentence(12..20).fake::<String>(),
            Sentence(12..20).fake::<String>()
        )
    }

    fn generate_special_instructions(assignment_type: &str, rng: &mut impl Rng) -> String {
        match assignment_type {
            "Programming" => {
                "All code must be properly version controlled with meaningful commit messages."
                    .to_string()
            }
            "Theoretical" => {
                "All mathematical notation must be properly formatted using LaTeX or similar."
                    .to_string()
            }
            "Analysis" => {
                "Include data visualizations and charts to support your analysis.".to_string()
            }
            "Design" => {
                "Provide mockups, wireframes, or architectural diagrams as appropriate.".to_string()
            }
            "Research" => "Follow proper academic citation format (APA, IEEE, or ACM).".to_string(),
            _ => format!(
                "Pay special attention to {}.",
                ["code quality", "documentation", "performance", "usability"]
                    [rng.random_range(0..4)]
            ),
        }
    }

    fn generate_file_format(rng: &mut impl Rng) -> String {
        let formats = [
            "ZIP or TAR.GZ",
            "Git repository URL",
            "PDF with embedded code",
            "Docker container",
            "Cloud deployment link",
        ];
        formats[rng.random_range(0..formats.len())].to_string()
    }

    fn generate_resources(rng: &mut impl Rng) -> String {
        let resources = [
            "Course slides and lecture recordings",
            "Recommended textbooks and academic papers",
            "Online documentation and API references",
            "Stack Overflow and technical forums",
            "GitHub repositories and open-source examples",
            "Industry blogs and technical articles",
        ];

        let selected: Vec<_> = resources.choose_multiple(rng, 4).cloned().collect();
        selected
            .iter()
            .map(|r| format!("- {}", r))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn generate_collaboration_policy(rng: &mut impl Rng) -> String {
        let policies = [
            "Individual work only - collaboration is not permitted for this assignment",
            "Pair programming allowed - submit one solution per pair with both names",
            "Group discussion encouraged - but each student must submit individual work",
            "Open collaboration - form teams of 2-3 students and submit group solutions",
        ];
        policies[rng.random_range(0..policies.len())].to_string()
    }
}

/// Template generator for study materials
#[cfg(feature = "dev-tools")]
pub struct StudyMaterialsTemplate;

#[cfg(feature = "dev-tools")]
impl StudyMaterialsTemplate {
    pub fn generate(material_type: &str, course: &Course, topic: &str) -> String {
        let mut seed = [0u8; 32];
        // Fill with some random-ish data
        for (i, byte) in seed.iter_mut().enumerate() {
            *byte = (i as u8).wrapping_mul(17).wrapping_add(42);
        }
        let mut rng = StdRng::from_seed(seed);

        match material_type {
            "Summary" => Self::generate_summary(course, topic, &mut rng),
            "Cheat Sheet" => Self::generate_cheat_sheet(course, topic, &mut rng),
            "Practice Problems" => Self::generate_practice_problems(course, topic, &mut rng),
            "Concept Map" => Self::generate_concept_map(course, topic, &mut rng),
            "Study Guide" => Self::generate_study_guide(course, topic, &mut rng),
            _ => Self::generate_generic_material(course, topic, &mut rng),
        }
    }

    fn generate_summary(course: &Course, topic: &str, rng: &mut impl Rng) -> String {
        let key_points = Self::generate_key_points(topic, rng);
        let examples = Self::generate_examples(topic, rng);
        let connections = Self::generate_topic_connections(rng);

        format!(
            r#"= {} - {} Summary
*Course*: {} \
*Topic*: {} \
*Last Updated*: {} \
*Study Time*: {} minutes

== Overview
{}

== Key Concepts
{}

== Important Definitions
{}

== Practical Examples
{}

== Common Patterns
{}

== Connections to Other Topics
{}

== Memory Techniques
{}

== Quick Review Checklist
{}

== Further Reading
{}

---
_Generated study summary - Review regularly for optimal retention_"#,
            course.code,
            topic,
            course.name,
            topic,
            chrono::Utc::now().format("%Y-%m-%d"),
            rng.random_range(15..45),
            Sentence(20..40).fake::<String>(),
            key_points,
            Self::generate_definitions(rng),
            examples,
            Self::generate_patterns(rng),
            connections,
            Self::generate_memory_techniques(rng),
            Self::generate_checklist(rng),
            Self::generate_references(rng)
        )
    }

    fn generate_cheat_sheet(course: &Course, topic: &str, rng: &mut impl Rng) -> String {
        let formulas = Self::generate_formulas(rng);
        let algorithms = Self::generate_algorithm_snippets(rng);
        let shortcuts = Self::generate_shortcuts(rng);

        format!(
            r#"= {} Quick Reference - {}
*Last Updated*: {}

== Essential Formulas
{}

== Key Algorithms
{}

== Common Patterns & Idioms
{}

== Troubleshooting Guide
{}

== Performance Tips
{}

== Keyboard Shortcuts
{}

== Useful Commands
{}

== Error Messages & Solutions
{}

---
*Pro Tip*: {} 📚"#,
            course.code,
            topic,
            chrono::Utc::now().format("%Y-%m-%d"),
            formulas,
            algorithms,
            Self::generate_code_patterns(rng),
            Self::generate_troubleshooting(rng),
            Self::generate_performance_tips(rng),
            shortcuts,
            Self::generate_commands(rng),
            Self::generate_error_solutions(rng),
            Self::generate_pro_tip(rng)
        )
    }

    fn generate_practice_problems(course: &Course, topic: &str, rng: &mut impl Rng) -> String {
        let easy_problems = Self::generate_problems("Easy", rng);
        let medium_problems = Self::generate_problems("Medium", rng);
        let hard_problems = Self::generate_problems("Hard", rng);

        format!(
            r#"= {} Practice Problems - {}
*Course*: {} \
*Difficulty Range*: Beginner to Advanced \
*Estimated Time*: {} hours

== Instructions
Work through these problems progressively. Start with Easy problems to build confidence, then advance to Medium and Hard problems. Solutions are provided at the end.

== Easy Problems (⭐)
{}

== Medium Problems (⭐⭐)
{}

== Hard Problems (⭐⭐⭐)
{}

== Challenge Problems (⭐⭐⭐⭐)
{}

== Solutions & Explanations
_Solutions available in separate document to prevent accidental spoilers_

== Additional Practice Resources
{}

== Study Tips
{}

---
*Remember*: Struggle is part of learning! Don't look at solutions too quickly."#,
            course.code,
            topic,
            course.name,
            rng.random_range(2..8),
            easy_problems,
            medium_problems,
            hard_problems,
            Self::generate_challenge_problems(rng),
            Self::generate_practice_resources(rng),
            Self::generate_study_tips(rng)
        )
    }

    fn generate_concept_map(course: &Course, topic: &str, rng: &mut impl Rng) -> String {
        let central_concepts = Self::generate_central_concepts(rng);
        let relationships = Self::generate_concept_relationships(rng);

        format!(
            r#"= {} Concept Map - {}
*Visual Learning Aid* \
*Course*: {}

== Central Concepts
{}

== Concept Hierarchy
```
Main Topic: {}
├── Fundamental Concepts
│   ├── {}
│   ├── {}
│   └── {}
├── Advanced Topics
│   ├── {}
│   ├── {}
│   └── {}
└── Practical Applications
    ├── {}
    ├── {}
    └── {}
```

== Concept Relationships
{}

== Learning Pathways
{}

== Cross-Subject Connections
{}

== Visual Mnemonics
{}

---
*Tip*: Use colors and symbols when studying to enhance memory retention!"#,
            course.code,
            topic,
            course.name,
            central_concepts,
            topic,
            Word().fake::<String>(),
            Word().fake::<String>(),
            Word().fake::<String>(),
            Words(2..3).fake::<Vec<String>>().join(" "),
            Words(2..3).fake::<Vec<String>>().join(" "),
            Words(2..3).fake::<Vec<String>>().join(" "),
            Words(2..4).fake::<Vec<String>>().join(" "),
            Words(2..4).fake::<Vec<String>>().join(" "),
            Words(2..4).fake::<Vec<String>>().join(" "),
            relationships,
            Self::generate_learning_pathways(rng),
            Self::generate_cross_connections(rng),
            Self::generate_mnemonics(rng)
        )
    }

    fn generate_study_guide(course: &Course, topic: &str, rng: &mut impl Rng) -> String {
        let schedule = Self::generate_study_schedule(rng);
        let objectives = Self::generate_learning_objectives(rng);

        format!(
            r#"= {} Study Guide - {}
*Course*: {} \
*Preparation Time*: {} days \
*Exam Weight*: {}%

== Learning Objectives
By the end of this study session, you should be able to:
{}

== Study Schedule
{}

== Topic Breakdown
{}

== Practice Strategy
{}

== Common Exam Questions
{}

== Last-Minute Review
{}

== Success Strategies
{}

== Stress Management
{}

== Post-Study Reflection
{}

---
*Good luck with your studies!* 🎯"#,
            course.code,
            topic,
            course.name,
            rng.random_range(3..14),
            rng.random_range(15..35),
            objectives,
            schedule,
            Self::generate_topic_breakdown(rng),
            Self::generate_practice_strategy(rng),
            Self::generate_exam_questions(rng),
            Self::generate_last_minute_review(rng),
            Self::generate_success_strategies(rng),
            Self::generate_stress_management(rng),
            Self::generate_reflection_questions(rng)
        )
    }

    fn generate_generic_material(course: &Course, topic: &str, rng: &mut impl Rng) -> String {
        format!(
            r#"= {} Study Material - {}
*Course*: {} \
*Topic*: {}

== Overview
{}

== Key Points
{}

== Examples
{}

== Practice Exercises
{}

== Additional Resources
{}"#,
            course.code,
            topic,
            course.name,
            topic,
            Sentence(15..30).fake::<String>(),
            Self::generate_key_points(topic, rng),
            Self::generate_examples(topic, rng),
            Self::generate_exercises(rng),
            Self::generate_resources_list(rng)
        )
    }

    // Helper methods for content generation
    fn generate_key_points(_topic: &str, rng: &mut impl Rng) -> String {
        (1..=rng.random_range(4..7))
            .map(|i| {
                format!(
                    "{}. **{}**: {}",
                    i,
                    Words(2..4).fake::<Vec<String>>().join(" "),
                    Sentence(10..20).fake::<String>()
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    fn generate_definitions(rng: &mut impl Rng) -> String {
        (1..=rng.random_range(3..6))
            .map(|_| {
                format!(
                    "- **{}**: {}",
                    Words(1..3).fake::<Vec<String>>().join(" "),
                    Sentence(8..15).fake::<String>()
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn generate_examples(_topic: &str, rng: &mut impl Rng) -> String {
        let example_types = [
            "Real-world application",
            "Code example",
            "Mathematical proof",
            "Case study",
        ];
        (1..=rng.random_range(2..4))
            .map(|i| {
                format!(
                    "=== Example {}: {}\n{}",
                    i,
                    example_types[rng.random_range(0..example_types.len())],
                    Sentence(15..25).fake::<String>()
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    fn generate_patterns(rng: &mut impl Rng) -> String {
        let patterns = [
            "Singleton pattern for resource management",
            "Observer pattern for event handling",
            "Factory pattern for object creation",
            "Strategy pattern for algorithm selection",
            "MVC pattern for separation of concerns",
        ];
        patterns
            .choose_multiple(rng, 3)
            .map(|p| format!("- {}", p))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn generate_topic_connections(rng: &mut impl Rng) -> String {
        (1..=rng.random_range(3..5))
            .map(|_| {
                format!(
                    "- **{}** → {}",
                    Words(2..4).fake::<Vec<String>>().join(" "),
                    Sentence(8..15).fake::<String>()
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn generate_memory_techniques(rng: &mut impl Rng) -> String {
        let techniques = [
            "Use the acronym **{}** to remember key concepts",
            "Visual metaphor: Think of {} as {}",
            "Memory palace: Associate {} with familiar locations",
            "Rhyme: \"{}\" helps recall the principle",
            "Story method: Create a narrative involving {}",
        ];

        techniques
            .choose_multiple(rng, 3)
            .map(|t| format!("- {}", t))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn generate_checklist(rng: &mut impl Rng) -> String {
        let items = [
            "Understand core definitions and terminology",
            "Can explain concepts to someone else",
            "Solved practice problems successfully",
            "Identified connections to other topics",
            "Created personal examples and analogies",
            "Reviewed common mistakes and pitfalls",
        ];

        items
            .choose_multiple(rng, 4)
            .map(|item| format!("- [ ] {}", item))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn generate_references(rng: &mut impl Rng) -> String {
        (1..=rng.random_range(3..5))
            .map(|_| {
                format!(
                    "- {}: \"{}\"",
                    Name().fake::<String>(),
                    Sentence(4..8).fake::<String>()
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn generate_formulas(rng: &mut impl Rng) -> String {
        let formulas = [
            "Time Complexity: O(n log n) for efficient sorting",
            "Space Complexity: O(1) for in-place algorithms",
            "Big-O notation: f(n) = O(g(n)) if ∃c,n₀ such that f(n) ≤ c·g(n)",
            "Recurrence: T(n) = 2T(n/2) + O(n)",
            "Probability: P(A∩B) = P(A)·P(B|A)",
        ];

        formulas
            .choose_multiple(rng, 3)
            .map(|f| format!("- {}", f))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn generate_algorithm_snippets(rng: &mut impl Rng) -> String {
        let snippets = [
            "```\nfor i in range(n):\n    # O(n) linear scan\n    process(data[i])\n```",
            "```\nwhile left < right:\n    # Binary search pattern\n    mid = (left + right) // 2\n```",
            "```\nfor i in range(len(arr)):\n    for j in range(i+1, len(arr)):\n        # O(n²) nested loop\n```",
        ];

        snippets
            .choose_multiple(rng, 2)
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    fn generate_code_patterns(rng: &mut impl Rng) -> String {
        let patterns = [
            "Early return pattern for cleaner code",
            "Guard clauses for input validation",
            "Builder pattern for complex object construction",
            "Null object pattern to avoid null checks",
            "Command pattern for undo/redo functionality",
        ];

        patterns
            .choose_multiple(rng, 3)
            .map(|p| format!("- {}", p))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn generate_troubleshooting(rng: &mut impl Rng) -> String {
        let issues = [
            "**Issue**: Code compiles but crashes → **Solution**: Check null pointer dereferences",
            "**Issue**: Infinite loop detected → **Solution**: Verify loop termination conditions",
            "**Issue**: Memory leak warnings → **Solution**: Ensure proper resource cleanup",
            "**Issue**: Performance degradation → **Solution**: Profile and optimize bottlenecks",
        ];

        issues
            .choose_multiple(rng, 3)
            .map(|i| format!("- {}", i))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn generate_performance_tips(rng: &mut impl Rng) -> String {
        let tips = [
            "Use appropriate data structures (HashMap vs Vec)",
            "Minimize memory allocations in hot paths",
            "Cache expensive computations when possible",
            "Profile before optimizing - measure twice, cut once",
            "Consider algorithmic improvements over micro-optimizations",
        ];

        tips.choose_multiple(rng, 3)
            .map(|t| format!("- {}", t))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn generate_shortcuts(rng: &mut impl Rng) -> String {
        let shortcuts = [
            "Ctrl+Shift+P: Command palette",
            "Ctrl+`: Toggle terminal",
            "Ctrl+B: Toggle sidebar",
            "F5: Start debugging",
            "Ctrl+Shift+F: Global search",
        ];

        shortcuts
            .choose_multiple(rng, 3)
            .map(|s| format!("- {}", s))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn generate_commands(rng: &mut impl Rng) -> String {
        let commands = [
            "`git status` - Check repository status",
            "`cargo test` - Run all tests",
            "`npm install` - Install dependencies",
            "`docker build .` - Build container image",
            "`grep -r \"pattern\" .` - Search in files",
        ];

        commands
            .choose_multiple(rng, 3)
            .map(|c| format!("- {}", c))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn generate_error_solutions(rng: &mut impl Rng) -> String {
        let errors = [
            "\"variable not found\" → Check spelling and scope",
            "\"type mismatch\" → Verify data types and conversions",
            "\"index out of bounds\" → Add bounds checking",
            "\"permission denied\" → Check file permissions and access rights",
        ];

        errors
            .choose_multiple(rng, 2)
            .map(|e| format!("- {}", e))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn generate_pro_tip(rng: &mut impl Rng) -> String {
        let tips = [
            "Practice spaced repetition for better retention",
            "Teach concepts to others to solidify understanding",
            "Create visual diagrams for complex relationships",
            "Use active recall instead of passive reading",
            "Build projects to apply theoretical knowledge",
        ];
        tips[rng.random_range(0..tips.len())].to_string()
    }

    fn generate_problems(difficulty: &str, rng: &mut impl Rng) -> String {
        let count = match difficulty {
            "Easy" => rng.random_range(4..7),
            "Medium" => rng.random_range(3..5),
            "Hard" => rng.random_range(2..4),
            _ => 3,
        };

        (1..=count)
            .map(|i| format!("{}. {}", i, Sentence(8..20).fake::<String>()))
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    fn generate_challenge_problems(_rng: &mut impl Rng) -> String {
        (1..=2)
            .map(|i| {
                format!(
                    "{}. **Challenge**: {}\n   *Hint*: {}",
                    i,
                    Sentence(10..25).fake::<String>(),
                    Sentence(6..12).fake::<String>()
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    fn generate_practice_resources(rng: &mut impl Rng) -> String {
        let resources = [
            "LeetCode problems tagged with relevant topics",
            "HackerRank skill assessments and practice",
            "Project Euler for mathematical programming challenges",
            "Codewars kata for diverse problem types",
            "Past exam questions from course materials",
        ];

        resources
            .choose_multiple(rng, 3)
            .map(|r| format!("- {}", r))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn generate_study_tips(rng: &mut impl Rng) -> String {
        let tips = [
            "Start with easier problems to build confidence",
            "Time yourself to simulate exam conditions",
            "Explain your solution approach out loud",
            "Review mistakes and understand why they occurred",
            "Practice regularly rather than cramming",
            "Join study groups for collaborative learning",
        ];

        tips.choose_multiple(rng, 4)
            .map(|t| format!("- {}", t))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn generate_central_concepts(rng: &mut impl Rng) -> String {
        (1..=rng.random_range(4..6))
            .map(|_| {
                format!(
                    "- **{}**: {}",
                    Words(2..4).fake::<Vec<String>>().join(" "),
                    Sentence(6..12).fake::<String>()
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn generate_concept_relationships(rng: &mut impl Rng) -> String {
        let relationships = [
            "is a prerequisite for",
            "builds upon",
            "contrasts with",
            "implements",
            "extends",
            "depends on",
        ];

        (1..=rng.random_range(3..5))
            .map(|_| {
                let rel = relationships[rng.random_range(0..relationships.len())];
                format!(
                    "- {} {} {}",
                    Words(2..3).fake::<Vec<String>>().join(" "),
                    rel,
                    Words(2..3).fake::<Vec<String>>().join(" ")
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn generate_learning_pathways(rng: &mut impl Rng) -> String {
        let pathways = [
            "**Beginner Path**: Start with fundamentals → Basic examples → Simple exercises",
            "**Intermediate Path**: Review basics → Advanced concepts → Complex problems",
            "**Advanced Path**: Skip to advanced topics → Research applications → Original projects",
        ];

        pathways
            .choose_multiple(rng, 2)
            .map(|p| format!("- {}", p))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn generate_cross_connections(rng: &mut impl Rng) -> String {
        let subjects = [
            "Mathematics",
            "Physics",
            "Economics",
            "Psychology",
            "Engineering",
        ];
        subjects
            .choose_multiple(rng, 3)
            .map(|s| format!("- **{}**: {}", s, Sentence(8..15).fake::<String>()))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn generate_mnemonics(rng: &mut impl Rng) -> String {
        let techniques = [
            "Color coding: Use different colors for different concept types",
            "Symbol system: Develop consistent symbols for common patterns",
            "Spatial organization: Group related concepts physically on paper",
            "Personal connections: Link abstract concepts to personal experiences",
        ];

        techniques
            .choose_multiple(rng, 2)
            .map(|t| format!("- {}", t))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn generate_learning_objectives(rng: &mut impl Rng) -> String {
        let objectives = [
            "Explain the fundamental principles and their applications",
            "Analyze complex problems using appropriate methodologies",
            "Synthesize information from multiple sources effectively",
            "Evaluate different approaches and justify choices",
            "Create original solutions to novel problems",
            "Demonstrate mastery through practical implementation",
        ];

        objectives
            .choose_multiple(rng, 4)
            .enumerate()
            .map(|(i, obj)| format!("{}. {}", i + 1, obj))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn generate_study_schedule(rng: &mut impl Rng) -> String {
        let days = ["Day 1", "Day 2", "Day 3", "Day 4", "Day 5"];
        let activities = [
            "Review lecture notes and key concepts",
            "Work through practice problems",
            "Create summary sheets and flashcards",
            "Form study group and discuss difficult topics",
            "Take practice exam under timed conditions",
            "Final review and relaxation",
        ];

        days.iter()
            .zip(activities.choose_multiple(rng, 5))
            .map(|(day, activity)| format!("- **{}**: {}", day, activity))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn generate_topic_breakdown(rng: &mut impl Rng) -> String {
        (1..=rng.random_range(4..6))
            .map(|i| {
                format!(
                    "=== Subtopic {}: {}\n- {}\n- {}\n- {}",
                    i,
                    Words(2..4).fake::<Vec<String>>().join(" "),
                    Sentence(6..12).fake::<String>(),
                    Sentence(6..12).fake::<String>(),
                    Sentence(6..12).fake::<String>()
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    fn generate_practice_strategy(rng: &mut impl Rng) -> String {
        let strategies = [
            "**Pomodoro Technique**: 25-minute focused study sessions with 5-minute breaks",
            "**Active Recall**: Test yourself without looking at notes",
            "**Spaced Repetition**: Review material at increasing intervals",
            "**Interleaving**: Mix different types of problems in study sessions",
            "**Elaborative Interrogation**: Ask yourself 'why' and 'how' questions",
        ];

        strategies
            .choose_multiple(rng, 3)
            .map(|s| format!("- {}", s))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn generate_exam_questions(rng: &mut impl Rng) -> String {
        (1..=rng.random_range(4..6))
            .map(|i| format!("{}. {}", i, Sentence(8..20).fake::<String>()))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn generate_last_minute_review(rng: &mut impl Rng) -> String {
        let items = [
            "Review summary sheets and flashcards",
            "Practice breathing exercises for stress management",
            "Check exam logistics (time, location, materials needed)",
            "Get adequate sleep - avoid all-nighters",
            "Eat a good breakfast and stay hydrated",
            "Arrive early to settle in and reduce anxiety",
        ];

        items
            .choose_multiple(rng, 4)
            .map(|item| format!("- {}", item))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn generate_success_strategies(rng: &mut impl Rng) -> String {
        let strategies = [
            "Form study groups with committed classmates",
            "Attend office hours and ask specific questions",
            "Use multiple learning modalities (visual, auditory, kinesthetic)",
            "Create a dedicated study environment free from distractions",
            "Set specific, achievable goals for each study session",
            "Track your progress and celebrate small wins",
        ];

        strategies
            .choose_multiple(rng, 4)
            .map(|s| format!("- {}", s))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn generate_stress_management(rng: &mut impl Rng) -> String {
        let techniques = [
            "**Deep Breathing**: 4-7-8 technique (inhale 4, hold 7, exhale 8)",
            "**Progressive Muscle Relaxation**: Tense and release muscle groups",
            "**Positive Visualization**: Imagine successful exam performance",
            "**Time Management**: Break large tasks into smaller, manageable chunks",
            "**Physical Exercise**: Regular walks or light exercise to reduce tension",
            "**Mindfulness**: Focus on present moment rather than future worries",
        ];

        techniques
            .choose_multiple(rng, 3)
            .map(|t| format!("- {}", t))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn generate_reflection_questions(rng: &mut impl Rng) -> String {
        let questions = [
            "What concepts do I understand well?",
            "Which areas need more practice?",
            "What study methods worked best for me?",
            "How can I improve my preparation for the next exam?",
            "What would I do differently if I could start over?",
            "How will I apply this knowledge in future courses?",
        ];

        questions
            .choose_multiple(rng, 4)
            .enumerate()
            .map(|(i, q)| format!("{}. {}", i + 1, q))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn generate_exercises(rng: &mut impl Rng) -> String {
        (1..=rng.random_range(3..5))
            .map(|i| format!("{}. {}", i, Sentence(8..15).fake::<String>()))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn generate_resources_list(rng: &mut impl Rng) -> String {
        let resources = [
            "Course textbook chapters 1-3",
            "Online tutorials and documentation",
            "Peer-reviewed academic papers",
            "Industry blogs and case studies",
            "Video lectures and recorded sessions",
            "Interactive coding platforms",
        ];

        resources
            .choose_multiple(rng, 4)
            .map(|r| format!("- {}", r))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// Template generator for course information
#[cfg(feature = "dev-tools")]
pub struct CourseInfoTemplate;

#[cfg(feature = "dev-tools")]
impl CourseInfoTemplate {
    pub fn generate(course: &Course) -> String {
        let mut seed = [0u8; 32];
        // Fill with some random-ish data
        for (i, byte) in seed.iter_mut().enumerate() {
            *byte = (i as u8).wrapping_mul(17).wrapping_add(42);
        }
        let mut rng = StdRng::from_seed(seed);

        let course_description = Self::generate_course_description(&mut rng);
        let prerequisites = Self::generate_prerequisites(&mut rng);
        let learning_outcomes = Self::generate_learning_outcomes(&mut rng);
        let assessment_methods = Self::generate_assessment_methods(&mut rng);
        let schedule = Self::generate_schedule(&mut rng);
        let resources = Self::generate_course_resources(&mut rng);

        format!(
            r#"= Course Information: {} - {}
*Course Code*: {} \
*Credits*: {} ECTS \
*Semester*: {} \
*Language*: English \
*Department*: DTU Compute

== Course Description
{}

== Prerequisites
{}

== Learning Outcomes
After completing this course, students will be able to:
{}

== Course Structure
{}

== Assessment Methods
{}

== Weekly Schedule
{}

== Course Materials
{}

== Teaching Staff
=== Course Responsible
- **{}** \
  Email: {} \
  Office Hours: {} \
  Research Interests: {}

=== Teaching Assistants
{}

== Important Dates
{}

== Course Policies
{}

== Student Support
{}

== Technology Requirements
{}

---
*Last Updated*: {} \
*Course Coordinator*: DTU Academic Affairs"#,
            course.code,
            course.name,
            course.code,
            course.credits,
            course.semester,
            course_description,
            prerequisites,
            learning_outcomes,
            Self::generate_course_structure(&mut rng),
            assessment_methods,
            schedule,
            resources,
            Self::generate_instructor_name(&mut rng),
            Self::generate_email(&mut rng),
            Self::generate_office_hours(&mut rng),
            Self::generate_research_interests(&mut rng),
            Self::generate_teaching_assistants(&mut rng),
            Self::generate_important_dates(&mut rng),
            Self::generate_course_policies(&mut rng),
            Self::generate_student_support(&mut rng),
            Self::generate_tech_requirements(&mut rng),
            chrono::Utc::now().format("%Y-%m-%d")
        )
    }

    fn generate_course_description(rng: &mut impl Rng) -> String {
        let course_types = [
            "This comprehensive course provides students with",
            "An intensive introduction to",
            "This advanced course covers",
            "A practical course that combines theory with",
            "An interdisciplinary course exploring",
        ];

        let focus_areas = [
            "fundamental principles and advanced applications",
            "cutting-edge research methods and industry practices",
            "theoretical foundations and practical implementations",
            "problem-solving techniques and real-world case studies",
            "computational methods and mathematical modeling",
        ];

        let outcomes = [
            "Students will develop strong analytical skills and practical expertise",
            "The course emphasizes hands-on experience and critical thinking",
            "Special attention is given to current industry trends and future developments",
            "Students work on individual and group projects throughout the semester",
            "The curriculum integrates academic research with practical applications",
        ];

        format!(
            "{} {}. {}.",
            course_types[rng.random_range(0..course_types.len())],
            focus_areas[rng.random_range(0..focus_areas.len())],
            outcomes[rng.random_range(0..outcomes.len())]
        )
    }

    fn generate_prerequisites(rng: &mut impl Rng) -> String {
        let formal_prereqs = [
            "Mathematics: Linear algebra and calculus (02105 or equivalent)",
            "Programming: Basic programming skills (02101 or equivalent)",
            "Statistics: Introduction to probability and statistics",
            "Computer Science: Data structures and algorithms (02102 or equivalent)",
        ];

        let recommended = [
            "Experience with Python or similar programming language",
            "Familiarity with Unix/Linux command line",
            "Basic understanding of software development practices",
            "Previous exposure to machine learning concepts is helpful but not required",
        ];

        format!(
            "=== Formal Prerequisites\n{}\n\n=== Recommended Background\n{}",
            {
                let count1 = rng.random_range(2..4);
                formal_prereqs
                    .choose_multiple(rng, count1)
                    .map(|p| format!("- {}", p))
                    .collect::<Vec<_>>()
                    .join("\n")
            },
            {
                let count2 = rng.random_range(2..3);
                recommended
                    .choose_multiple(rng, count2)
                    .map(|r| format!("- {}", r))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        )
    }

    fn generate_learning_outcomes(rng: &mut impl Rng) -> String {
        let cognitive_outcomes = [
            "Analyze complex problems using appropriate theoretical frameworks",
            "Evaluate different solution approaches and justify design decisions",
            "Synthesize information from multiple sources to solve novel problems",
        ];

        let practical_outcomes = [
            "Implement efficient algorithms and data structures",
            "Design and develop software systems following best practices",
            "Apply debugging and testing techniques to ensure code quality",
            "Use version control and collaborative development tools effectively",
        ];

        let communication_outcomes = [
            "Present technical findings clearly to both technical and non-technical audiences",
            "Write comprehensive technical documentation and reports",
            "Collaborate effectively in team-based software development projects",
        ];

        format!(
            "=== Knowledge and Understanding\n{}\n\n=== Skills\n{}\n\n=== Competences\n{}",
            cognitive_outcomes
                .choose_multiple(rng, 2)
                .enumerate()
                .map(|(i, o)| format!("{}. {}", i + 1, o))
                .collect::<Vec<_>>()
                .join("\n"),
            practical_outcomes
                .choose_multiple(rng, 3)
                .enumerate()
                .map(|(i, o)| format!("{}. {}", i + 1, o))
                .collect::<Vec<_>>()
                .join("\n"),
            communication_outcomes
                .choose_multiple(rng, 2)
                .enumerate()
                .map(|(i, o)| format!("{}. {}", i + 1, o))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }

    fn generate_course_structure(rng: &mut impl Rng) -> String {
        let structures = [
            "The course is organized into weekly modules, each focusing on a specific topic with accompanying hands-on exercises.",
            "Lectures are complemented by practical laboratory sessions where students apply theoretical concepts.",
            "The course follows a project-based learning approach with regular milestones and peer reviews.",
            "Weekly seminars combine theoretical presentations with practical coding workshops.",
        ];

        let components = [
            "- **Lectures**: 2 hours per week covering theoretical foundations",
            "- **Exercises**: 2 hours per week of guided problem-solving sessions",
            "- **Laboratory Work**: Hands-on programming and experimentation",
            "- **Project Work**: Individual and group assignments throughout the semester",
            "- **Seminars**: Student presentations and peer discussion sessions",
        ];

        format!(
            "{}\n\n{}",
            structures[rng.random_range(0..structures.len())],
            components
                .choose_multiple(rng, 4)
                .map(|c| c.to_string())
                .collect::<Vec<_>>()
                .join("\n")
        )
    }

    fn generate_assessment_methods(rng: &mut impl Rng) -> String {
        let methods = [
            (
                "Written Exam",
                rng.random_range(40..60),
                "Final comprehensive examination covering all course topics",
            ),
            (
                "Programming Assignments",
                rng.random_range(25..35),
                "Individual coding projects demonstrating practical skills",
            ),
            (
                "Group Project",
                rng.random_range(15..25),
                "Collaborative software development project",
            ),
            (
                "Class Participation",
                rng.random_range(5..15),
                "Active engagement in discussions and peer review",
            ),
        ];

        let total_weight: i32 = methods.iter().map(|(_, weight, _)| *weight).sum();
        let adjustment = 100 - total_weight;
        let mut adjusted_methods = methods;
        adjusted_methods[0].1 += adjustment; // Add adjustment to largest component

        format!(
            "{}\n\n=== Grading Scale\n- A: 90-100% (Excellent)\n- B: 80-89% (Very Good)\n- C: 70-79% (Good)\n- D: 60-69% (Satisfactory)\n- E: 50-59% (Sufficient)\n- F: <50% (Fail)",
            adjusted_methods
                .iter()
                .map(|(method, weight, desc)| format!("- **{}** ({}%): {}", method, weight, desc))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }

    fn generate_schedule(rng: &mut impl Rng) -> String {
        let days = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday"];
        let times = ["08:00-10:00", "10:00-12:00", "13:00-15:00", "15:00-17:00"];
        let locations = [
            "Building 321, Room A23",
            "Building 324, Lab B14",
            "Building 322, Auditorium C",
            "Building 325, Computer Lab",
        ];

        let schedule_items = [
            (
                "Lecture",
                "Weekly theoretical presentations and concept introduction",
            ),
            ("Exercise Session", "Guided problem-solving and Q&A"),
            ("Laboratory", "Hands-on programming and experimentation"),
            (
                "Office Hours",
                "Individual consultation with instructor and TAs",
            ),
        ];

        schedule_items
            .choose_multiple(rng, 3)
            .map(|(activity, desc)| {
                let day = days[rng.random_range(0..days.len())];
                let time = times[rng.random_range(0..times.len())];
                let location = locations[rng.random_range(0..locations.len())];
                format!(
                    "- **{}** {}: {} - {}\n  _{}_",
                    day, time, activity, location, desc
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    fn generate_course_resources(rng: &mut impl Rng) -> String {
        let textbooks = [
            "\"Introduction to Algorithms\" by Cormen, Leiserson, Rivest, and Stein",
            "\"Design Patterns\" by Gang of Four",
            "\"Clean Code\" by Robert C. Martin",
            "\"The Pragmatic Programmer\" by Hunt and Thomas",
        ];

        let online_resources = [
            "Course website with lecture slides and assignments",
            "DTU Learn platform for discussions and submissions",
            "Recorded lecture videos available on DTU Inside",
            "Online coding environment for practical exercises",
        ];

        let software = [
            "Python 3.8+ with scientific computing libraries",
            "Git for version control and collaboration",
            "Visual Studio Code or similar IDE",
            "Docker for containerized development environments",
        ];

        format!(
            "=== Required Textbooks\n{}\n\n=== Online Resources\n{}\n\n=== Software and Tools\n{}",
            textbooks
                .choose_multiple(rng, 2)
                .map(|t| format!("- {}", t))
                .collect::<Vec<_>>()
                .join("\n"),
            online_resources
                .choose_multiple(rng, 3)
                .map(|r| format!("- {}", r))
                .collect::<Vec<_>>()
                .join("\n"),
            software
                .choose_multiple(rng, 3)
                .map(|s| format!("- {}", s))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }

    fn generate_instructor_name(rng: &mut impl Rng) -> String {
        let first_names = [
            "Lars", "Anne", "Henrik", "Maria", "Thomas", "Kirsten", "Michael", "Susanne",
        ];
        let last_names = [
            "Hansen",
            "Nielsen",
            "Jensen",
            "Andersen",
            "Pedersen",
            "Christensen",
            "Larsen",
            "Sørensen",
        ];

        format!(
            "{} {}",
            first_names[rng.random_range(0..first_names.len())],
            last_names[rng.random_range(0..last_names.len())]
        )
    }

    fn generate_email(rng: &mut impl Rng) -> String {
        let domains = ["@dtu.dk", "@compute.dtu.dk"];
        format!(
            "{}{}",
            (0..rng.random_range(4..8))
                .map(|_| char::from(b'a' + rng.random_range(0..26)))
                .collect::<String>(),
            domains[rng.random_range(0..domains.len())]
        )
    }

    fn generate_office_hours(rng: &mut impl Rng) -> String {
        let days = ["Tuesday", "Wednesday", "Thursday"];
        let times = ["14:00-16:00", "10:00-12:00", "13:00-15:00"];
        format!(
            "{} {}",
            days[rng.random_range(0..days.len())],
            times[rng.random_range(0..times.len())]
        )
    }

    fn generate_research_interests(rng: &mut impl Rng) -> String {
        let interests = [
            "Machine Learning and AI",
            "Software Engineering",
            "Distributed Systems",
            "Human-Computer Interaction",
            "Computer Graphics",
            "Data Science",
            "Cybersecurity",
            "Algorithms and Complexity",
            "Database Systems",
        ];
        interests
            .choose_multiple(rng, 2)
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn generate_teaching_assistants(rng: &mut impl Rng) -> String {
        (1..=rng.random_range(2..4))
            .map(|_| {
                let name = Self::generate_instructor_name(rng);
                let email = Self::generate_email(rng);
                format!("- **{}** ({})", name, email)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn generate_important_dates(rng: &mut impl Rng) -> String {
        let dates = [
            "Course Start: Week 1 of semester",
            "Assignment 1 Due: Week 4",
            "Midterm Exam: Week 8",
            "Project Presentations: Week 12",
            "Assignment 2 Due: Week 13",
            "Final Exam: Exam period (Week 15-17)",
        ];

        dates
            .choose_multiple(rng, 5)
            .map(|d| format!("- {}", d))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn generate_course_policies(rng: &mut impl Rng) -> String {
        let policies = [
            "**Attendance**: Regular attendance is expected but not mandatory for lectures",
            "**Late Submissions**: 10% penalty per day late, maximum 3 days",
            "**Academic Integrity**: All work must be original; collaboration guidelines apply",
            "**Re-examination**: Available for students who fail the regular exam",
            "**Special Accommodations**: Contact course coordinator for accessibility needs",
        ];

        policies
            .choose_multiple(rng, 4)
            .map(|p| format!("- {}", p))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn generate_student_support(rng: &mut impl Rng) -> String {
        let support_options = [
            "**Study Groups**: Encouraged and facilitated through DTU Learn platform",
            "**Tutoring**: Available through DTU's academic support services",
            "**Counseling**: DTU Student Counseling Service for academic and personal support",
            "**Technical Support**: IT Help Desk for software and platform issues",
            "**Language Support**: Danish and English writing assistance available",
        ];

        support_options
            .choose_multiple(rng, 3)
            .map(|s| format!("- {}", s))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn generate_tech_requirements(rng: &mut impl Rng) -> String {
        let requirements = [
            "Laptop or desktop computer with Windows, macOS, or Linux",
            "Minimum 8GB RAM, 256GB storage space available",
            "Stable internet connection for online resources and video lectures",
            "Webcam and microphone for virtual sessions and presentations",
            "Access to DTU network (VPN available for off-campus access)",
        ];

        requirements
            .choose_multiple(rng, 4)
            .map(|r| format!("- {}", r))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// Get lecture topics for a specific course
#[cfg(feature = "dev-tools")]
pub fn get_lecture_topics(course_code: &str) -> Vec<String> {
    match course_code {
        "02101" => vec![
            "Introduction to Programming".to_string(),
            "Variables and Data Types".to_string(),
            "Control Structures".to_string(),
            "Functions and Modules".to_string(),
            "Data Structures".to_string(),
            "File I/O".to_string(),
            "Error Handling".to_string(),
            "Object-Oriented Programming".to_string(),
            "Testing and Debugging".to_string(),
            "Best Practices".to_string(),
        ],
        "02102" => vec![
            "Algorithm Analysis".to_string(),
            "Arrays and Lists".to_string(),
            "Stacks and Queues".to_string(),
            "Linked Lists".to_string(),
            "Trees and Binary Trees".to_string(),
            "Hash Tables".to_string(),
            "Sorting Algorithms".to_string(),
            "Searching Algorithms".to_string(),
            "Graph Algorithms".to_string(),
            "Dynamic Programming".to_string(),
        ],
        "02105" => vec![
            "Advanced Data Structures".to_string(),
            "Graph Theory".to_string(),
            "Network Flows".to_string(),
            "String Algorithms".to_string(),
            "Computational Geometry".to_string(),
            "Approximation Algorithms".to_string(),
            "Randomized Algorithms".to_string(),
            "Parallel Algorithms".to_string(),
            "Advanced Topics".to_string(),
            "Research Frontiers".to_string(),
        ],
        "02180" => vec![
            "Introduction to AI".to_string(),
            "Search Algorithms".to_string(),
            "Knowledge Representation".to_string(),
            "Machine Learning Basics".to_string(),
            "Neural Networks".to_string(),
            "Natural Language Processing".to_string(),
            "Computer Vision".to_string(),
            "Robotics".to_string(),
            "Ethics in AI".to_string(),
            "Future of AI".to_string(),
        ],
        _ => vec![
            "Introduction".to_string(),
            "Fundamental Concepts".to_string(),
            "Core Theory".to_string(),
            "Practical Applications".to_string(),
            "Advanced Topics".to_string(),
            "Case Studies".to_string(),
            "Integration".to_string(),
            "Optimization".to_string(),
            "Best Practices".to_string(),
            "Future Directions".to_string(),
        ],
    }
}
