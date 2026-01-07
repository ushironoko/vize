#!/usr/bin/env node --experimental-strip-types
/**
 * Generate expected outputs from Vue's official compiler.
 *
 * Usage:
 *   node --experimental-strip-types scripts/generate-expected.ts
 *   node --experimental-strip-types scripts/generate-expected.ts sfc/script-setup
 */

import * as fs from 'node:fs'
import * as path from 'node:path'
import { parse as parseTOML } from '@iarna/toml'
import { compileScript, compileTemplate, parse as parseSFC } from '@vue/compiler-sfc'

interface TestCase {
  name: string
  input: string
  options?: Record<string, any>
}

interface TestFixture {
  mode?: string
  cases: TestCase[]
}

const fixturesDir = path.join(import.meta.dirname!, '..', 'tests', 'fixtures')
const expectedDir = path.join(import.meta.dirname!, '..', 'tests', 'expected')

function loadFixture(filePath: string): TestFixture {
  const content = fs.readFileSync(filePath, 'utf-8')
  return parseTOML(content) as any
}

function compileSFC(source: string, filename: string = 'test.vue', isTS: boolean = false): string {
  const { descriptor, errors } = parseSFC(source, { filename })

  if (errors.length > 0) {
    return `Parse errors: ${errors.map(e => e.message).join(', ')}`
  }

  // Detect if script is TypeScript
  const scriptLang = descriptor.scriptSetup?.lang || descriptor.script?.lang
  const actualIsTS = isTS || scriptLang === 'ts' || scriptLang === 'tsx'

  let code = ''

  // Compile template if present
  let templateResult: ReturnType<typeof compileTemplate> | null = null
  if (descriptor.template) {
    const bindings = descriptor.scriptSetup ? compileScript(descriptor, {
      id: filename,
      templateOptions: {}
    }).bindings : undefined

    templateResult = compileTemplate({
      source: descriptor.template.content,
      filename,
      id: filename,
      compilerOptions: {
        bindingMetadata: bindings,
        mode: 'module',
      },
    })
  }

  // Compile script setup
  if (descriptor.scriptSetup) {
    const result = compileScript(descriptor, {
      id: filename,
      inlineTemplate: true,
      templateOptions: templateResult ? {
        compilerOptions: {
          mode: 'module',
        }
      } : undefined,
    })
    code = result.content
  } else if (descriptor.script) {
    // Options API - rewrite export default
    const scriptContent = descriptor.script.content
    const rewritten = scriptContent.replace(
      /export\s+default\s+\{/,
      'const _sfc_main = {'
    )

    if (templateResult) {
      code = templateResult.code + '\n' + rewritten + '\n_sfc_main.render = _sfc_render\nexport default _sfc_main'
    } else {
      code = rewritten + '\nexport default _sfc_main'
    }
  } else if (templateResult) {
    // Template only
    code = templateResult.code
  }

  return code
}

function formatSnapFile(cases: Array<{name: string, input: string, output: string, css?: string}>): string {
  let content = ''

  for (const testCase of cases) {
    content += `===\n`
    content += `name: ${testCase.name}\n`
    content += `options: sfc\n`
    content += `--- INPUT ---\n`
    content += testCase.input.trim() + '\n'
    content += `--- OUTPUT ---\n`
    content += testCase.output.trim() + '\n'
    if (testCase.css) {
      content += `--- CSS ---\n`
      content += testCase.css.trim() + '\n'
    }
  }

  return content
}

function processFixture(fixturePath: string, outputPath: string) {
  console.log(`Processing: ${path.relative(fixturesDir, fixturePath)}`)

  const fixture = loadFixture(fixturePath)
  const results: Array<{name: string, input: string, output: string}> = []

  for (const testCase of fixture.cases) {
    try {
      const output = compileSFC(testCase.input, 'test.vue')
      results.push({
        name: testCase.name,
        input: testCase.input,
        output,
      })
    } catch (error) {
      console.error(`  Error compiling "${testCase.name}":`, error)
      results.push({
        name: testCase.name,
        input: testCase.input,
        output: `Compile error: ${error}`,
      })
    }
  }

  // Ensure output directory exists
  fs.mkdirSync(path.dirname(outputPath), { recursive: true })

  // Write snap file
  const content = formatSnapFile(results)
  fs.writeFileSync(outputPath, content)
  console.log(`  Written: ${path.relative(expectedDir, outputPath)} (${results.length} cases)`)
}

function main() {
  const args = process.argv.slice(2)

  // If specific fixture is provided
  if (args.length > 0) {
    for (const arg of args) {
      const fixturePath = path.join(fixturesDir, `${arg}.toml`)
      const outputPath = path.join(expectedDir, `${arg}.snap`)

      if (!fs.existsSync(fixturePath)) {
        console.error(`Fixture not found: ${fixturePath}`)
        continue
      }

      processFixture(fixturePath, outputPath)
    }
    return
  }

  // Process all SFC fixtures
  const sfcFixturesDir = path.join(fixturesDir, 'sfc')
  if (fs.existsSync(sfcFixturesDir)) {
    const files = fs.readdirSync(sfcFixturesDir).filter(f => f.endsWith('.toml'))
    for (const file of files) {
      const fixturePath = path.join(sfcFixturesDir, file)
      const outputPath = path.join(expectedDir, 'sfc', file.replace('.toml', '.snap'))
      processFixture(fixturePath, outputPath)
    }
  }
}

main()
