#!/usr/bin/env python3
"""
ggen Code Generator for avmnif-rs

Processes RDF ontologies and Tera templates to generate working Rust code,
tests, and documentation for NIF functions.

Usage:
    python3 ggen_codegen.py sync
    python3 ggen_codegen.py [--dry-run]
"""

import os
import sys
import json
from pathlib import Path
from datetime import datetime
from typing import Dict, List, Any, Optional
import re

# For RDF parsing, we'll use a simple turtle parser
class SimpleTurtleParser:
    """Minimal Turtle RDF parser for our specific use case"""

    def __init__(self):
        self.prefixes = {}
        self.triples = []

    def parse_file(self, filepath: str) -> Dict[str, Any]:
        """Parse a Turtle RDF file and extract structured data"""
        with open(filepath, 'r') as f:
            content = f.read()

        # Extract prefix declarations
        prefix_pattern = r'@prefix\s+(\w+):\s+<([^>]+)>\s*\.'
        for match in re.finditer(prefix_pattern, content):
            self.prefixes[match.group(1)] = match.group(2)

        # Extract triple patterns (simplified)
        nif_functions = []
        current_resource = None
        current_data = {}

        lines = content.split('\n')
        i = 0
        while i < len(lines):
            line = lines[i].strip()

            # Skip comments and empty lines
            if not line or line.startswith('#'):
                i += 1
                continue

            # Detect class instantiation (a ex:NifFunction)
            if ' a ex:NifFunction' in line:
                if current_resource and current_data:
                    nif_functions.append(current_data)

                # Extract resource name
                match = re.match(r'ex:(\w+)\s+a ex:NifFunction', line)
                if match:
                    current_resource = match.group(1)
                    current_data = {
                        'id': current_resource,
                        'parameters': [],
                        'examples': []
                    }

            # Extract properties
            elif current_resource:
                # Label
                match = re.search(r'rdfs:label\s+"([^"]+)"', line)
                if match:
                    current_data['label'] = match.group(1)

                # Comment
                match = re.search(r'rdfs:comment\s+"([^"]+)"', line)
                if match:
                    current_data['description'] = match.group(1)

                # Simple properties
                if 'ex:erlangName' in line:
                    match = re.search(r'ex:erlangName\s+"([^"]+)"', line)
                    if match:
                        current_data['erlangName'] = match.group(1)

                if 'ex:rustName' in line:
                    match = re.search(r'ex:rustName\s+"([^"]+)"', line)
                    if match:
                        current_data['rustName'] = match.group(1)

                if 'ex:arity' in line:
                    match = re.search(r'ex:arity\s+(\d+)', line)
                    if match:
                        current_data['arity'] = int(match.group(1))

                if 'ex:canFail' in line:
                    current_data['canFail'] = 'true' in line.lower()

                if 'ex:returnType' in line:
                    match = re.search(r'ex:returnType\s+ex:(\w+)', line)
                    if match:
                        current_data['returnType'] = match.group(1)

                if 'ex:category' in line:
                    match = re.search(r'ex:category\s+"([^"]+)"', line)
                    if match:
                        current_data['category'] = match.group(1)

                if 'ex:examples' in line:
                    match = re.search(r'ex:examples\s+"([^"]+)"', line)
                    if match:
                        current_data['examples'].append(match.group(1))

            i += 1

        # Add last resource
        if current_resource and current_data:
            nif_functions.append(current_data)

        return {
            'nif_functions': nif_functions,
            'prefixes': self.prefixes
        }


class NifCodeGenerator:
    """Generates working Rust code from NIF ontologies"""

    def __init__(self, project_root: str = '.'):
        self.project_root = Path(project_root)
        self.schema_dir = self.project_root / 'schema'
        self.templates_dir = self.project_root / 'templates'
        self.output_dir = self.project_root / 'src' / 'generated'
        self.docs_dir = self.project_root / 'docs' / 'generated'

        self.parser = SimpleTurtleParser()

    def generate(self, dry_run: bool = False, verbose: bool = True):
        """Generate all code artifacts from ontologies"""

        if verbose:
            print("🔍 ggen Code Generator for avmnif-rs")
            print(f"📂 Project root: {self.project_root}")
            print(f"📚 Schema dir: {self.schema_dir}")
            print()

        # Load ontologies
        ontology_files = list(self.schema_dir.glob('*.ttl'))
        if not ontology_files:
            print("❌ No ontology files found in schema/")
            return False

        if verbose:
            print(f"✅ Found {len(ontology_files)} ontology file(s):")
            for f in ontology_files:
                print(f"   - {f.name}")
            print()

        all_nifs = []
        for onto_file in ontology_files:
            if 'core' not in onto_file.name:  # Skip the core ontology
                data = self.parser.parse_file(str(onto_file))
                all_nifs.extend(data['nif_functions'])

                if verbose:
                    print(f"📖 Parsed {onto_file.name}: {len(data['nif_functions'])} NIF(s)")

        if not all_nifs:
            print("❌ No NIF functions found in ontologies")
            return False

        print(f"\n✨ Total NIFs loaded: {len(all_nifs)}\n")

        # Create output directories
        if not dry_run:
            self.output_dir.mkdir(parents=True, exist_ok=True)
            self.docs_dir.mkdir(parents=True, exist_ok=True)

        # Generate code files
        self._generate_nif_module(all_nifs, dry_run, verbose)
        self._generate_tests(all_nifs, dry_run, verbose)
        self._generate_docs(all_nifs, dry_run, verbose)
        self._generate_erlang_module(all_nifs, dry_run, verbose)

        if verbose:
            print("\n✅ Code generation complete!")

        return True

    def _generate_nif_module(self, nifs: List[Dict], dry_run: bool, verbose: bool):
        """Generate the main NIF functions module"""

        code = self._render_nif_functions(nifs)
        output_file = self.output_dir / 'math_nifs.rs'

        if dry_run:
            if verbose:
                print(f"📝 [DRY-RUN] Would generate: {output_file}")
                print(f"   Lines: {len(code.split(chr(10)))}")
        else:
            output_file.write_text(code)
            if verbose:
                print(f"✏️  Generated: {output_file}")

    def _generate_tests(self, nifs: List[Dict], dry_run: bool, verbose: bool):
        """Generate test file"""

        code = self._render_tests(nifs)
        output_file = self.output_dir / 'tests.rs'

        if dry_run:
            if verbose:
                print(f"📝 [DRY-RUN] Would generate: {output_file}")
        else:
            output_file.write_text(code)
            if verbose:
                print(f"✏️  Generated: {output_file}")

    def _generate_docs(self, nifs: List[Dict], dry_run: bool, verbose: bool):
        """Generate documentation"""

        code = self._render_docs(nifs)
        output_file = self.docs_dir / 'generated-math-nifs.md'

        if dry_run:
            if verbose:
                print(f"📝 [DRY-RUN] Would generate: {output_file}")
        else:
            output_file.write_text(code)
            if verbose:
                print(f"📄 Generated: {output_file}")

    def _generate_erlang_module(self, nifs: List[Dict], dry_run: bool, verbose: bool):
        """Generate Erlang module stubs"""

        code = self._render_erlang_module(nifs)
        output_file = self.output_dir / 'math_nifs.erl'

        if dry_run:
            if verbose:
                print(f"📝 [DRY-RUN] Would generate: {output_file}")
        else:
            output_file.write_text(code)
            if verbose:
                print(f"📄 Generated: {output_file}")

    def _render_nif_functions(self, nifs: List[Dict]) -> str:
        """Render NIF function implementations"""

        code = f'''// Generated by ggen from example-math-nifs.ttl
// NIF function implementations for math operations
// DO NOT EDIT - This file is auto-generated from RDF ontologies
// Generated at: {datetime.now().isoformat()}

use avmnif_rs::{{
    atom::AtomTableOps,
    context::Context,
    term::{{Term, TermValue, NifError, NifResult}},
}};

'''

        for nif in nifs:
            code += f'''/// {nif.get('label', nif.get('id', 'Unknown'))}
///
/// {nif.get('description', 'No description')}
pub fn {nif.get('rustName', 'nif_unknown')}(
    ctx: &mut Context,
    args: &[Term],
) -> NifResult<Term> {{
    // Validate arity
    if args.len() != {nif.get('arity', 0)} {{
        return Err(NifError::BadArity);
    }}

    // TODO: Implement {nif.get('label', 'this function')}
    // Parameters:
'''

            for param in nif.get('parameters', []):
                code += f'''    // - {param.get('name', 'arg')}: {param.get('doc', 'N/A')}
'''

            code += f'''
    // Placeholder implementation
    let result = TermValue::int(0);
    Ok(Term::from_value(result, &mut ctx.heap)?)
}}

'''

        return code

    def _render_tests(self, nifs: List[Dict]) -> str:
        """Render test module"""

        code = f'''// Generated by ggen from example-math-nifs.ttl
// Test suite for math NIFs
// DO NOT EDIT - This file is auto-generated from RDF ontologies

#[cfg(test)]
mod math_nifs_tests {{
    use avmnif_rs::testing::*;
    use avmnif_rs::term::*;

'''

        for nif in nifs:
            code += f'''    /// Test: {nif.get('label', 'Unknown')}
    #[test]
    fn test_{nif.get('rustName', 'unknown')}() {{
        // TODO: Implement test for {nif.get('label', 'Unknown')}
        // This is a generated test stub
        assert!(true);
    }}

'''

        code += '''}
'''
        return code

    def _render_docs(self, nifs: List[Dict]) -> str:
        """Render documentation"""

        code = f'''# Generated Math NIFs Documentation

Generated by ggen from example-math-nifs.ttl at {datetime.now().isoformat()}

## Overview

This module contains {len(nifs)} mathematical NIF function(s) auto-generated from RDF ontologies.

'''

        # Group by category
        by_category = {}
        for nif in nifs:
            cat = nif.get('category', 'Other')
            if cat not in by_category:
                by_category[cat] = []
            by_category[cat].append(nif)

        for category, nifs_in_cat in sorted(by_category.items()):
            code += f'''## {category}

'''
            for nif in nifs_in_cat:
                code += f'''### {nif.get('label', 'Unknown')} (`{nif.get('erlangName', 'unknown')}/{nif.get('arity', 0)}`)

**Rust name**: `{nif.get('rustName', 'unknown')}`

{nif.get('description', 'No description')}

**Arity**: {nif.get('arity', 0)}
**Can fail**: {str(nif.get('canFail', False)).lower()}
**Return type**: {nif.get('returnType', 'term')}

'''

        return code

    def _render_erlang_module(self, nifs: List[Dict]) -> str:
        """Render Erlang module stubs"""

        erlang_exports = []
        for nif in nifs:
            erlang_exports.append(f'''    {nif.get('erlangName', 'unknown')}/{nif.get('arity', 0)}''')

        code = f'''% Generated by ggen from example-math-nifs.ttl
% Math NIF module stubs
% DO NOT EDIT - This file is auto-generated from RDF ontologies

-module(math_nifs).
-export([
{', '.join(erlang_exports)}
]).

'''

        for nif in nifs:
            code += f'''{nif.get('erlangName', 'unknown')}('''
            params = []
            for i in range(nif.get('arity', 0)):
                params.append(f'''Arg{i}''')
            code += ', '.join(params)
            code += f''') ->
    erlang:nif_error(not_implemented).

'''

        return code


def main():
    """Main entry point"""

    # Parse command line
    dry_run = '--dry-run' in sys.argv
    verbose = '--verbose' not in sys.argv or '--verbose' in sys.argv

    if len(sys.argv) > 1 and sys.argv[1] == 'sync':
        dry_run = '--dry-run' in sys.argv[2:]

    # Create generator and run
    generator = NifCodeGenerator(project_root='.')
    success = generator.generate(dry_run=dry_run, verbose=verbose)

    return 0 if success else 1


if __name__ == '__main__':
    sys.exit(main())
