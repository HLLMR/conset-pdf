# ML Ruleset Compiler

## Overview

The ML Ruleset Compiler provides **ML-assisted profile generation** from abstract transcripts. It uses LLM APIs to automatically propose extraction profiles for specification documents, drawing templates, and equipment submittals.

**Note**: The compiler has been significantly improved with structured prompts, shape-based placeholders, repetition metrics, and line grouping. These improvements are integrated into the current implementation.

## Architecture

### Compile-Validate Loop

The compiler implements a robust **compile-validate loop**:

1. **Propose**: LLM analyzes abstract transcript and proposes a profile
2. **Validate**: Profile is validated against measurable gates
3. **Re-prompt**: If validation fails, LLM is re-prompted with failure report
4. **Store**: Accepted profiles are stored in the profile registry

### Privacy-Preserving

The compiler operates on **abstract transcripts** where sensitive content has been replaced with structural tokens. This ensures:
- No sensitive data sent to LLM
- Layout and structure preserved
- Deterministic tokenization

## Usage

### Basic Usage

```typescript
import { createAPIRulesetCompiler, sanitizeTranscript, createTranscriptExtractor } from '@conset-pdf/core';
import { PrivacyMode } from '@conset-pdf/core';

// Extract and sanitize transcript
const extractor = createTranscriptExtractor();
const transcript = await extractor.extractTranscript('path/to/file.pdf');
const { abstractTranscript } = sanitizeTranscript(transcript, {
  privacyMode: PrivacyMode.STRICT_STRUCTURE_ONLY,
});

// Create compiler
const compiler = createAPIRulesetCompiler({
  apiKey: process.env.LLM_API_KEY,
  model: 'gpt-4',
});

// Propose spec profile
const candidate = await compiler.proposeSpecProfile({
  abstractTranscript,
  docTypeHint: 'spec',
});

// Check validation
if (candidate.validation.valid) {
  console.log('Profile generated successfully!');
  console.log('Confidence:', candidate.validation.confidence);
} else {
  console.log('Validation issues:', candidate.validation.issues);
}
```

### Configuration

The compiler can be configured via environment variables or constructor options:

```typescript
const compiler = createAPIRulesetCompiler({
  apiUrl: process.env.LLM_API_URL || 'https://api.openai.com/v1/chat/completions',
  apiKey: process.env.LLM_API_KEY,
  model: process.env.LLM_MODEL || 'gpt-4',
  temperature: 0.3,
  maxTokens: 4000,
}, 3); // maxAttempts = 3
```

**Environment Variables:**
- `LLM_API_URL`: API endpoint URL (default: OpenAI)
- `LLM_API_KEY`: API key for authentication
- `LLM_MODEL`: Model identifier (default: 'gpt-4')

## Profile Types

### Spec Profile

Proposes extraction profiles for specification documents:

```typescript
const candidate = await compiler.proposeSpecProfile({
  abstractTranscript,
  docTypeHint: 'spec',
  context: {
    notes: 'This is a MasterFormat specification document',
    characteristics: ['Section-based', 'Hierarchical headings'],
  },
});
```

**Generated Profile Includes:**
- Page model (size, rotation)
- Regions (header, footer, body bands)
- Outline structure (heading levels with patterns)
- Optional table extraction strategies

### Sheet Template Profile

Proposes extraction profiles for drawing/sheet templates:

```typescript
const candidate = await compiler.proposeSheetTemplateProfile({
  abstractTranscript,
  docTypeHint: 'drawing',
});
```

**Generated Profile Includes:**
- Page model constraints
- Title block configuration (bbox, field definitions)
- Standard blocks (revision block, notes, etc.)
- Schedule definitions

### Equipment Submittal Profile

Proposes extraction profiles for equipment submittal documents:

```typescript
const candidate = await compiler.proposeEquipmentSubmittalProfile({
  abstractTranscript,
  docTypeHint: 'equipmentSubmittal',
});
```

**Generated Profile Includes:**
- Unit structure patterns (cover block, unit report)
- Header block ROIs with field definitions
- Table definitions for performance data

## Validation Gates

Profiles are validated against measurable gates:

**For Spec Profiles:**
- Header/footer coverage ≥ 80%
- Heading hierarchy consistency ≥ 70%
- Body band exclusion ≥ 95%

**For Sheet Template Profiles:**
- Schedule structure match ≥ 80%

**For Equipment Submittal Profiles:**
- Table structure match ≥ 80%

## Error Handling

The compiler handles various error scenarios:

- **API Errors**: Throws descriptive errors for API failures
- **Parsing Errors**: Re-prompts with error details
- **Validation Failures**: Re-prompts with validation issues
- **Max Attempts**: Returns last profile with validation issues if max attempts reached

## Testing

Tests use mocked LLM responses to verify:
- Profile parsing from LLM responses
- Markdown code block handling
- Retry logic on validation failures
- Error handling

See `tests/transcript/ml/apiCompiler.test.ts` for examples.

## Integration with Profile Registry

Generated profiles can be stored in the profile registry:

```typescript
import { ProfileRegistry } from '@conset-pdf/core';

const registry = new ProfileRegistry();
const candidate = await compiler.proposeSpecProfile({ abstractTranscript });

if (candidate.validation.valid) {
  await registry.saveProfile(candidate.profile, {
    profileId: 'generated-spec-001',
    name: 'Auto-generated Spec Profile',
    version: '1.0.0',
    source: 'ml-generated',
  });
}
```

## Limitations

- **Requires API Key**: LLM API access required
- **Cost**: Each profile generation consumes API tokens
- **Validation Dependency**: Full validation requires original transcript (not anonymized)
- **Model Quality**: Profile quality depends on LLM model capabilities

## Future Enhancements

- **Local Models**: Support for local LLM models (Ollama, etc.)
- **Fine-tuning**: Fine-tuned models for specific document types
- **Batch Processing**: Generate profiles for multiple documents
- **Profile Refinement**: Interactive refinement of generated profiles

## See Also

- [TRANSCRIPT_ARCHITECTURE.md](./TRANSCRIPT_ARCHITECTURE.md) - Transcript system overview
- [MIGRATION_V3.md](./MIGRATION_V3.md) - Migration guide
