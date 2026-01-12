<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted, shallowRef } from 'vue';
import * as monaco from 'monaco-editor';

// Script tag attributes
const SCRIPT_TAG_ATTRS = [
  { label: 'setup', insertText: 'setup', detail: 'Enable <script setup> syntax' },
  { label: 'vapor', insertText: 'vapor', detail: 'Enable Vapor mode compilation' },
  { label: 'lang="ts"', insertText: 'lang="ts"', detail: 'Use TypeScript' },
  { label: 'lang="tsx"', insertText: 'lang="tsx"', detail: 'Use TSX' },
  { label: 'generic', insertText: 'generic="${1:T}"', detail: 'Define generic type parameters' },
];

// Template tag attributes
const TEMPLATE_TAG_ATTRS = [
  { label: 'lang="pug"', insertText: 'lang="pug"', detail: 'Use Pug template syntax' },
];

// Style tag attributes
const STYLE_TAG_ATTRS = [
  { label: 'scoped', insertText: 'scoped', detail: 'Scope styles to this component' },
  { label: 'module', insertText: 'module', detail: 'Enable CSS modules' },
  { label: 'lang="scss"', insertText: 'lang="scss"', detail: 'Use SCSS' },
  { label: 'lang="less"', insertText: 'lang="less"', detail: 'Use Less' },
];

// Vue compiler macros for completion
const VUE_COMPILER_MACROS = [
  { label: 'defineProps', insertText: 'defineProps<${1:Props}>()', detail: 'Define component props' },
  { label: 'defineEmits', insertText: 'defineEmits<${1:Emits}>()', detail: 'Define component emits' },
  { label: 'defineExpose', insertText: 'defineExpose({ $1 })', detail: 'Expose component methods' },
  { label: 'defineOptions', insertText: 'defineOptions({ $1 })', detail: 'Define component options' },
  { label: 'defineSlots', insertText: 'defineSlots<${1:Slots}>()', detail: 'Define typed slots' },
  { label: 'defineModel', insertText: 'defineModel<${1:T}>(${2})', detail: 'Define v-model binding' },
  { label: 'withDefaults', insertText: 'withDefaults(defineProps<${1:Props}>(), {\n  $2\n})', detail: 'Props with defaults' },
];

// Vue reactivity APIs
const VUE_REACTIVITY_APIS = [
  { label: 'ref', insertText: 'ref($1)', detail: 'Create a reactive reference' },
  { label: 'reactive', insertText: 'reactive({ $1 })', detail: 'Create a reactive object' },
  { label: 'computed', insertText: 'computed(() => $1)', detail: 'Create a computed value' },
  { label: 'watch', insertText: 'watch($1, ($2) => {\n  $3\n})', detail: 'Watch reactive source' },
  { label: 'watchEffect', insertText: 'watchEffect(() => {\n  $1\n})', detail: 'Run effect immediately' },
  { label: 'toRef', insertText: 'toRef($1, \'$2\')', detail: 'Create ref from reactive property' },
  { label: 'toRefs', insertText: 'toRefs($1)', detail: 'Convert reactive to refs' },
];

// Hover documentation for compiler macros
interface HoverDoc {
  signature: string;
  description: string;
  examples: string[];
  docUrl: string;
  since?: string;
  deprecated?: string;
  seeAlso?: string[];
}

const COMPILER_MACRO_DOCS: Record<string, HoverDoc> = {
  defineProps: {
    signature: 'defineProps<T>(): Readonly<T>\ndefineProps(options: object): Readonly<Props>',
    description: `**defineProps** はコンポーネントの props を定義するコンパイラマクロです。

\`<script setup>\` 内でのみ使用可能で、インポートなしで直接呼び出せます。

**2つの宣言スタイル:**
- **型ベース宣言** (推奨): TypeScript の型パラメータで props を定義
- **ランタイム宣言**: オブジェクトで props オプションを定義

コンパイル時に適切なランタイムコードに変換されます。`,
    examples: [
      `// 型ベース宣言 (推奨)
const props = defineProps<{
  title: string
  count?: number
}>()`,
      `// ランタイム宣言
const props = defineProps({
  title: { type: String, required: true },
  count: { type: Number, default: 0 }
})`,
      `// デフォルト値付き (withDefaults と併用)
const props = withDefaults(defineProps<{
  msg?: string
  labels?: string[]
}>(), {
  msg: 'hello',
  labels: () => ['one', 'two']
})`
    ],
    docUrl: 'https://vuejs.org/api/sfc-script-setup.html#defineprops-defineemits',
    since: 'Vue 3.0',
    seeAlso: ['withDefaults', 'defineEmits']
  },

  defineEmits: {
    signature: 'defineEmits<T>(): T\ndefineEmits(options: string[] | object): EmitFn',
    description: `**defineEmits** はコンポーネントが発行できるイベントを定義するコンパイラマクロです。

\`<script setup>\` 内でのみ使用可能で、インポートなしで直接呼び出せます。

型ベース宣言により、イベント名とペイロードの型安全性を確保できます。`,
    examples: [
      `// 型ベース宣言 (推奨)
const emit = defineEmits<{
  (e: 'change', id: number): void
  (e: 'update', value: string): void
}>()`,
      `// Vue 3.3+ 簡略構文
const emit = defineEmits<{
  change: [id: number]
  update: [value: string]
}>()`,
      `// ランタイム宣言
const emit = defineEmits(['change', 'update'])

// バリデーション付き
const emit = defineEmits({
  change: (id: number) => id > 0,
  update: null // バリデーションなし
})`
    ],
    docUrl: 'https://vuejs.org/api/sfc-script-setup.html#defineprops-defineemits',
    since: 'Vue 3.0',
    seeAlso: ['defineProps']
  },

  defineExpose: {
    signature: 'defineExpose(exposed: Record<string, any>): void',
    description: `**defineExpose** は親コンポーネントに公開するプロパティ/メソッドを明示的に指定するコンパイラマクロです。

\`<script setup>\` を使用するコンポーネントはデフォルトで閉じられており、テンプレート参照や \`$parent\` チェーン経由でアクセスできません。

\`defineExpose\` を使用して、公開する値を明示的に指定する必要があります。`,
    examples: [
      `// 基本的な使用法
const count = ref(0)
const increment = () => count.value++

defineExpose({
  count,
  increment
})`,
      `// 親コンポーネントからのアクセス
// <ChildComponent ref="child" />
const child = ref<InstanceType<typeof ChildComponent>>()
child.value?.increment()`
    ],
    docUrl: 'https://vuejs.org/api/sfc-script-setup.html#defineexpose',
    since: 'Vue 3.0',
    seeAlso: ['ref', 'Template Refs']
  },

  defineOptions: {
    signature: 'defineOptions(options: ComponentOptions): void',
    description: `**defineOptions** は \`<script setup>\` 内でコンポーネントオプションを直接宣言するコンパイラマクロです。

\`inheritAttrs\` や \`name\` など、\`<script setup>\` で直接表現できないオプションを設定する場合に使用します。

**注意:** props、emits、expose、slots は defineOptions では設定できません。専用のマクロを使用してください。`,
    examples: [
      `// コンポーネント名の設定
defineOptions({
  name: 'MyComponent'
})`,
      `// 属性の継承を無効化
defineOptions({
  inheritAttrs: false
})`,
      `// 複数のオプション
defineOptions({
  name: 'CustomButton',
  inheritAttrs: false,
  customOption: 'value' // カスタムオプション
})`
    ],
    docUrl: 'https://vuejs.org/api/sfc-script-setup.html#defineoptions',
    since: 'Vue 3.3'
  },

  defineSlots: {
    signature: 'defineSlots<T>(): Readonly<T>',
    description: `**defineSlots** はスロットの型を定義するコンパイラマクロです。

スロット名と props の型チェックを有効にし、\`useSlots()\` の戻り値の型を推論します。

現在は型宣言のみ対応しており、ランタイム宣言は提供されていません。`,
    examples: [
      `// スロットの型定義
const slots = defineSlots<{
  default(props: { msg: string }): any
  header(props: { title: string }): any
}>()`,
      `// 複雑なスロット props
defineSlots<{
  item(props: {
    item: Item
    index: number
  }): any
}>()`
    ],
    docUrl: 'https://vuejs.org/api/sfc-script-setup.html#defineslots',
    since: 'Vue 3.3',
    seeAlso: ['useSlots']
  },

  defineModel: {
    signature: 'defineModel<T>(name?: string, options?: object): ModelRef<T>',
    description: `**defineModel** は双方向バインディング (v-model) を実装するためのコンパイラマクロです。

内部的に prop と対応する \`update:xxx\` イベントを宣言し、それを直接変更可能な ref として返します。

\`v-model\` の実装を大幅に簡略化できます。`,
    examples: [
      `// 基本的な v-model
const modelValue = defineModel<string>()
// 親: <Child v-model="value" />`,
      `// 名前付き v-model
const title = defineModel<string>('title')
// 親: <Child v-model:title="title" />`,
      `// オプション付き
const count = defineModel<number>('count', {
  default: 0,
  required: true
})`,
      `// 変換オプション (Vue 3.4+)
const [modelValue, modifiers] = defineModel<string>({
  set(value) {
    if (modifiers.capitalize) {
      return value.charAt(0).toUpperCase() + value.slice(1)
    }
    return value
  }
})`
    ],
    docUrl: 'https://vuejs.org/api/sfc-script-setup.html#definemodel',
    since: 'Vue 3.4',
    seeAlso: ['defineProps', 'defineEmits']
  },

  withDefaults: {
    signature: 'withDefaults<T>(props: T, defaults: Partial<T>): T',
    description: `**withDefaults** は型ベースの \`defineProps\` にデフォルト値を提供するコンパイラマクロです。

型ベースの \`defineProps\` 宣言ではデフォルト値を直接指定できないため、このマクロを使用します。

**注意:** オブジェクトや配列のデフォルト値はファクトリ関数で返す必要があります。`,
    examples: [
      `// 基本的な使用法
const props = withDefaults(defineProps<{
  msg?: string
  count?: number
}>(), {
  msg: 'hello',
  count: 0
})`,
      `// 配列/オブジェクトのデフォルト値
const props = withDefaults(defineProps<{
  items?: string[]
  config?: { debug: boolean }
}>(), {
  items: () => ['default'],
  config: () => ({ debug: false })
})`
    ],
    docUrl: 'https://vuejs.org/api/sfc-script-setup.html#default-props-values-when-using-type-declaration',
    since: 'Vue 3.0',
    seeAlso: ['defineProps']
  }
};

// Hover documentation for Vue reactivity APIs
const VUE_API_DOCS: Record<string, HoverDoc> = {
  ref: {
    signature: 'ref<T>(value: T): Ref<UnwrapRef<T>>',
    description: `**ref** はリアクティブでミュータブルな参照オブジェクトを作成します。

\`.value\` プロパティを通じて内部の値にアクセス・変更できます。

テンプレート内では自動的にアンラップされ、\`.value\` なしでアクセスできます。`,
    examples: [
      `const count = ref(0)
console.log(count.value) // 0
count.value++
console.log(count.value) // 1`,
      `// 型注釈付き
const name = ref<string | null>(null)`,
      `// テンプレート内では自動アンラップ
// <template>{{ count }}</template>`
    ],
    docUrl: 'https://vuejs.org/api/reactivity-core.html#ref',
    since: 'Vue 3.0',
    seeAlso: ['reactive', 'computed', 'shallowRef']
  },

  reactive: {
    signature: 'reactive<T extends object>(target: T): UnwrapNestedRefs<T>',
    description: `**reactive** はオブジェクトのリアクティブプロキシを返します。

オブジェクト全体がディープリアクティブになり、ネストされたプロパティも追跡されます。

**注意:** プリミティブ値には使用できません。\`ref\` を使用してください。`,
    examples: [
      `const state = reactive({
  count: 0,
  nested: { value: 'hello' }
})

// 直接アクセス (.value 不要)
state.count++
state.nested.value = 'world'`,
      `// 分割代入するとリアクティビティが失われる
// BAD: const { count } = state
// GOOD: const { count } = toRefs(state)`
    ],
    docUrl: 'https://vuejs.org/api/reactivity-core.html#reactive',
    since: 'Vue 3.0',
    seeAlso: ['ref', 'toRefs', 'shallowReactive']
  },

  computed: {
    signature: 'computed<T>(getter: () => T): ComputedRef<T>\ncomputed<T>(options: { get: () => T, set: (v: T) => void }): WritableComputedRef<T>',
    description: `**computed** は計算された ref を作成します。

getter 関数の戻り値を追跡し、依存関係が変更されたときのみ再計算されます。

結果はキャッシュされ、依存関係が変更されるまで再計算されません。`,
    examples: [
      `// 読み取り専用の computed
const count = ref(1)
const doubled = computed(() => count.value * 2)`,
`// 書き込み可能な computed
const firstName = ref('John')
const lastName = ref('Doe')
const fullName = computed({
  get: () => firstName.value + ' ' + lastName.value,
  set: (val) => {
    [firstName.value, lastName.value] = val.split(' ')
  }
})`
    ],
    docUrl: 'https://vuejs.org/api/reactivity-core.html#computed',
    since: 'Vue 3.0',
    seeAlso: ['ref', 'watch', 'watchEffect']
  },

  watch: {
    signature: 'watch<T>(source: WatchSource<T>, callback: WatchCallback<T>, options?: WatchOptions): StopHandle',
    description: `**watch** は1つ以上のリアクティブなデータソースを監視し、ソースが変更されたときにコールバック関数を呼び出します。

\`watchEffect\` と異なり、明示的に監視対象を指定する必要があります。

デフォルトで lazy（遅延評価）で、ソースが変更されたときのみコールバックが呼ばれます。`,
    examples: [
`// 単一の ref を監視
const count = ref(0)
watch(count, (newVal, oldVal) => {
  console.log('count changed: ' + oldVal + ' -> ' + newVal)
})`,
`// 複数のソースを監視
watch([firstName, lastName], ([newFirst, newLast]) => {
  console.log('Name: ' + newFirst + ' ' + newLast)
})`,
      `// deep オプション
watch(state, (newState) => {
  console.log('state changed deeply')
}, { deep: true })`,
      `// immediate オプション (初期実行)
watch(source, callback, { immediate: true })`
    ],
    docUrl: 'https://vuejs.org/api/reactivity-core.html#watch',
    since: 'Vue 3.0',
    seeAlso: ['watchEffect', 'computed']
  },

  watchEffect: {
    signature: 'watchEffect(effect: (onCleanup: OnCleanup) => void, options?: WatchEffectOptions): StopHandle',
    description: `**watchEffect** は副作用を即座に実行しながら、その依存関係をリアクティブに追跡します。

依存関係が変更されるたびに副作用が再実行されます。

\`watch\` と異なり、監視対象を明示的に指定する必要がなく、コールバック内でアクセスしたリアクティブな値がすべて追跡されます。`,
    examples: [
`const count = ref(0)

// 即座に実行され、count が変更されるたびに再実行
watchEffect(() => {
  console.log('count is: ' + count.value)
})`,
      `// クリーンアップ関数
watchEffect((onCleanup) => {
  const timer = setInterval(() => {}, 1000)
  onCleanup(() => clearInterval(timer))
})`,
      `// flush オプション (DOM 更新後に実行)
watchEffect(callback, { flush: 'post' })`
    ],
    docUrl: 'https://vuejs.org/api/reactivity-core.html#watcheffect',
    since: 'Vue 3.0',
    seeAlso: ['watch', 'watchPostEffect', 'watchSyncEffect']
  },

  toRef: {
    signature: 'toRef<T, K extends keyof T>(object: T, key: K): ToRef<T[K]>',
    description: `**toRef** はリアクティブオブジェクトのプロパティへの ref を作成します。

作成された ref はソースプロパティと同期されます。ソースを変更すると ref も更新され、逆も同様です。

\`reactive\` オブジェクトのプロパティを別の関数に渡す際にリアクティビティを維持するために使用します。`,
    examples: [
      `const state = reactive({
  foo: 1,
  bar: 2
})

// fooRef は state.foo と同期する
const fooRef = toRef(state, 'foo')

fooRef.value++
console.log(state.foo) // 2

state.foo++
console.log(fooRef.value) // 3`
    ],
    docUrl: 'https://vuejs.org/api/reactivity-utilities.html#toref',
    since: 'Vue 3.0',
    seeAlso: ['toRefs', 'ref']
  },

  toRefs: {
    signature: 'toRefs<T extends object>(object: T): ToRefs<T>',
    description: `**toRefs** はリアクティブオブジェクトを通常のオブジェクトに変換します。各プロパティは元のプロパティへの ref になります。

\`reactive\` オブジェクトを分割代入してもリアクティビティを失わないようにするために使用します。

Composition API の composable 関数から値を返す際によく使用されます。`,
    examples: [
      `const state = reactive({
  foo: 1,
  bar: 2
})

// 分割代入してもリアクティブ
const { foo, bar } = toRefs(state)

foo.value++
console.log(state.foo) // 2`,
      `// Composable からの return
function useFeature() {
  const state = reactive({
    x: 0,
    y: 0
  })
  // リアクティブな値として返す
  return toRefs(state)
}`
    ],
    docUrl: 'https://vuejs.org/api/reactivity-utilities.html#torefs',
    since: 'Vue 3.0',
    seeAlso: ['toRef', 'reactive']
  }
};

// Hover documentation for Vue directives
const VUE_DIRECTIVE_DOCS: Record<string, HoverDoc> = {
  'v-if': {
    signature: 'v-if="expression"',
    description: `**v-if** は条件付きでテンプレートブロックをレンダリングするディレクティブです。

式が truthy の場合のみ、要素とその内容がレンダリングされます。

\`v-else\` や \`v-else-if\` と組み合わせて使用できます。

**注意:** \`v-if\` はトグル時に要素を完全に作成/破棄します。頻繁なトグルには \`v-show\` を検討してください。`,
    examples: [
      `<div v-if="isVisible">表示される内容</div>`,
      `<template v-if="condition">
  <h1>タイトル</h1>
  <p>コンテンツ</p>
</template>`,
      `<div v-if="type === 'A'">A</div>
<div v-else-if="type === 'B'">B</div>
<div v-else>その他</div>`
    ],
    docUrl: 'https://vuejs.org/guide/essentials/conditional.html',
    since: 'Vue 2.0',
    seeAlso: ['v-else', 'v-else-if', 'v-show']
  },

  'v-else': {
    signature: 'v-else',
    description: `**v-else** は \`v-if\` または \`v-else-if\` の "else ブロック" を表します。

値は不要で、直前の兄弟要素に \`v-if\` または \`v-else-if\` が必要です。`,
    examples: [
      `<div v-if="isLoggedIn">ログイン済み</div>
<div v-else>ログインしてください</div>`,
      `<template v-if="items.length">
  <ul>...</ul>
</template>
<p v-else>アイテムがありません</p>`
    ],
    docUrl: 'https://vuejs.org/guide/essentials/conditional.html#v-else',
    since: 'Vue 2.0',
    seeAlso: ['v-if', 'v-else-if']
  },

  'v-else-if': {
    signature: 'v-else-if="expression"',
    description: `**v-else-if** は \`v-if\` の "else if ブロック" を表します。

チェーンして複数の条件分岐を表現できます。

直前の兄弟要素に \`v-if\` または \`v-else-if\` が必要です。`,
    examples: [
      `<div v-if="score >= 90">A</div>
<div v-else-if="score >= 80">B</div>
<div v-else-if="score >= 70">C</div>
<div v-else>D</div>`
    ],
    docUrl: 'https://vuejs.org/guide/essentials/conditional.html#v-else-if',
    since: 'Vue 2.0',
    seeAlso: ['v-if', 'v-else']
  },

  'v-for': {
    signature: 'v-for="(item, index) in items" :key="item.id"',
    description: `**v-for** は配列やオブジェクトに基づいて要素のリストをレンダリングするディレクティブです。

**構文:**
- \`item in items\` - 配列の各要素
- \`(item, index) in items\` - 要素とインデックス
- \`(value, key) in object\` - オブジェクトの値とキー
- \`(value, key, index) in object\` - 値、キー、インデックス
- \`n in 10\` - 数値範囲 (1 から n)

**重要:** パフォーマンスと正確な DOM 更新のため、\`:key\` 属性を必ず指定してください。`,
    examples: [
      `<li v-for="item in items" :key="item.id">
  {{ item.name }}
</li>`,
      `<li v-for="(item, index) in items" :key="item.id">
  {{ index }}: {{ item.name }}
</li>`,
      `<div v-for="(value, key) in object" :key="key">
  {{ key }}: {{ value }}
</div>`,
      `<!-- template で複数要素をグループ化 -->
<template v-for="item in items" :key="item.id">
  <h2>{{ item.title }}</h2>
  <p>{{ item.body }}</p>
</template>`
    ],
    docUrl: 'https://vuejs.org/guide/essentials/list.html',
    since: 'Vue 2.0',
    seeAlso: ['v-if', 'key']
  },

  'v-model': {
    signature: 'v-model="data"\nv-model:argument="data"\nv-model.modifier="data"',
    description: `**v-model** はフォーム入力要素やコンポーネントに双方向バインディングを作成します。

内部的には value prop と input イベント（または対応するもの）の糖衣構文です。

**対応する要素:**
- \`<input>\` - value + input
- \`<textarea>\` - value + input
- \`<select>\` - value + change
- コンポーネント - modelValue + update:modelValue

**修飾子:**
- \`.lazy\` - change イベントで同期
- \`.number\` - 数値に変換
- \`.trim\` - 空白をトリム`,
    examples: [
      `<!-- テキスト入力 -->
<input v-model="message" />`,
      `<!-- チェックボックス -->
<input type="checkbox" v-model="checked" />`,
      `<!-- 複数選択 -->
<select v-model="selected" multiple>
  <option value="a">A</option>
  <option value="b">B</option>
</select>`,
      `<!-- 修飾子 -->
<input v-model.lazy="msg" />
<input v-model.number="age" type="number" />
<input v-model.trim="name" />`,
      `<!-- コンポーネント (Vue 3.4+) -->
<Child v-model="value" />
<Child v-model:title="title" />`
    ],
    docUrl: 'https://vuejs.org/guide/essentials/forms.html',
    since: 'Vue 2.0',
    seeAlso: ['defineModel', 'defineProps', 'defineEmits']
  },

  'v-on': {
    signature: 'v-on:event="handler"\n@event="handler"\n@event.modifier="handler"',
    description: `**v-on** (省略形: @) は DOM イベントをリッスンし、発火時にハンドラを実行します。

**イベント修飾子:**
- \`.stop\` - event.stopPropagation()
- \`.prevent\` - event.preventDefault()
- \`.capture\` - キャプチャモードで追加
- \`.self\` - event.target が要素自身の場合のみ
- \`.once\` - 最大1回
- \`.passive\` - パッシブリスナー

**キー修飾子:**
- \`.enter\`, \`.tab\`, \`.delete\`, \`.esc\`, \`.space\`
- \`.up\`, \`.down\`, \`.left\`, \`.right\`
- \`.ctrl\`, \`.alt\`, \`.shift\`, \`.meta\``,
    examples: [
      `<!-- メソッドハンドラ -->
<button @click="handleClick">クリック</button>`,
      `<!-- インライン式 -->
<button @click="count++">+1</button>`,
      `<!-- 引数付き -->
<button @click="say('hello')">Hello</button>`,
      `<!-- イベント修飾子 -->
<form @submit.prevent="onSubmit">...</form>
<a @click.stop.prevent="doThat">...</a>`,
      `<!-- キー修飾子 -->
<input @keyup.enter="submit" />
<input @keydown.ctrl.s="save" />`
    ],
    docUrl: 'https://vuejs.org/guide/essentials/event-handling.html',
    since: 'Vue 2.0'
  },

  'v-bind': {
    signature: 'v-bind:attribute="expression"\n:attribute="expression"\nv-bind="object"',
    description: `**v-bind** (省略形: :) は1つ以上の属性またはコンポーネント prop を動的にバインドします。

**修飾子:**
- \`.prop\` - DOM プロパティとしてバインド
- \`.camel\` - kebab-case を camelCase に変換
- \`.attr\` - 強制的に DOM 属性としてバインド

**特殊なバインディング:**
- \`:class\` - オブジェクトまたは配列構文をサポート
- \`:style\` - オブジェクトまたは配列構文をサポート`,
    examples: [
      `<!-- 属性バインディング -->
<img :src="imageSrc" :alt="imageAlt" />`,
      `<!-- クラスバインディング -->
<div :class="{ active: isActive, 'error': hasError }"></div>
<div :class="[activeClass, errorClass]"></div>`,
      `<!-- スタイルバインディング -->
<div :style="{ color: textColor, fontSize: size + 'px' }"></div>`,
      `<!-- オブジェクト展開 -->
<component v-bind="$attrs"></component>
<Child v-bind="props"></Child>`
    ],
    docUrl: 'https://vuejs.org/guide/essentials/class-and-style.html',
    since: 'Vue 2.0',
    seeAlso: ['v-on', 'v-model']
  },

  'v-slot': {
    signature: 'v-slot:slotName="slotProps"\n#slotName="slotProps"',
    description: `**v-slot** (省略形: #) は名前付きスロットまたはスコープ付きスロットを受け取ることを示します。

コンポーネントまたは \`<template>\` 要素でのみ使用可能です。

**注意:** デフォルトスロットは \`#default\` または \`v-slot\` で参照できます。`,
    examples: [
      `<!-- 名前付きスロット -->
<template #header>
  <h1>ヘッダー</h1>
</template>`,
      `<!-- スコープ付きスロット -->
<template #item="{ item, index }">
  {{ index }}: {{ item.name }}
</template>`,
      `<!-- 省略記法 -->
<MyComponent #default="{ data }">
  {{ data }}
</MyComponent>`,
      `<!-- 動的スロット名 -->
<template #[slotName]="slotProps">
  ...
</template>`
    ],
    docUrl: 'https://vuejs.org/guide/components/slots.html',
    since: 'Vue 2.6',
    seeAlso: ['defineSlots', 'useSlots']
  },

  'v-show': {
    signature: 'v-show="expression"',
    description: `**v-show** は式の truthy/falsy に基づいて要素の可視性を切り替えます。

CSS の \`display\` プロパティを使用するため、要素は常に DOM に存在します。

**v-if との違い:**
- \`v-show\` は CSS で切り替え（初期レンダリングコストが高い）
- \`v-if\` は DOM を作成/破棄（トグルコストが高い）

頻繁にトグルする場合は \`v-show\`、条件がほとんど変わらない場合は \`v-if\` を推奨。`,
    examples: [
      `<div v-show="isVisible">
  常に DOM に存在、display で切り替え
</div>`
    ],
    docUrl: 'https://vuejs.org/guide/essentials/conditional.html#v-show',
    since: 'Vue 2.0',
    seeAlso: ['v-if']
  },

  'v-pre': {
    signature: 'v-pre',
    description: `**v-pre** は要素とそのすべての子要素のコンパイルをスキップします。

mustache タグを生のテキストとして表示する場合に使用します。

コンパイルをスキップするため、ディレクティブのない大きな量のノードでパフォーマンスが向上します。`,
    examples: [
      `<span v-pre>{{ これはそのまま表示される }}</span>`
    ],
    docUrl: 'https://vuejs.org/api/built-in-directives.html#v-pre',
    since: 'Vue 2.0'
  },

  'v-once': {
    signature: 'v-once',
    description: `**v-once** は要素とコンポーネントを一度だけレンダリングし、以降の更新をスキップします。

静的コンテンツの最適化に使用します。

子コンポーネントや v-for で使用する場合、サブツリー全体に影響します。`,
    examples: [
      `<span v-once>初期値: {{ initialValue }}</span>`,
      `<!-- 静的リスト -->
<ul v-once>
  <li v-for="item in staticItems" :key="item.id">
    {{ item.name }}
  </li>
</ul>`
    ],
    docUrl: 'https://vuejs.org/api/built-in-directives.html#v-once',
    since: 'Vue 2.0',
    seeAlso: ['v-memo']
  },

  'v-memo': {
    signature: 'v-memo="[dep1, dep2, ...]"',
    description: `**v-memo** はテンプレートのサブツリーをメモ化します。

依存関係配列内の値が変更されない限り、サブツリーの更新をスキップします。

\`v-for\` と組み合わせて、大きなリストの部分的な再レンダリングを最適化できます。`,
    examples: [
      `<div v-memo="[valueA, valueB]">
  <!-- valueA または valueB が変更された場合のみ更新 -->
</div>`,
      `<!-- v-for での最適化 -->
<div v-for="item in list" :key="item.id" v-memo="[item.selected]">
  <p>ID: {{ item.id }} - selected: {{ item.selected }}</p>
  <!-- item.selected が変更された場合のみ更新 -->
</div>`
    ],
    docUrl: 'https://vuejs.org/api/built-in-directives.html#v-memo',
    since: 'Vue 3.2',
    seeAlso: ['v-once', 'v-for']
  },

  'v-cloak': {
    signature: 'v-cloak',
    description: `**v-cloak** は Vue インスタンスが準備完了するまで要素を非表示にするために使用します。

CSS と組み合わせて、コンパイル前の mustache タグが表示されるのを防ぎます。

Vue がマウントされると自動的に削除されます。`,
    examples: [
      `<!-- CSS -->
<style>
[v-cloak] { display: none; }
</style>

<!-- HTML -->
<div v-cloak>
  {{ message }}
</div>`
    ],
    docUrl: 'https://vuejs.org/api/built-in-directives.html#v-cloak',
    since: 'Vue 2.0'
  },

  'v-html': {
    signature: 'v-html="rawHtml"',
    description: `**v-html** は要素の innerHTML を更新します。

**⚠️ セキュリティ警告:**
信頼できないコンテンツに \`v-html\` を使用すると、XSS 攻撃につながる可能性があります。

ユーザー提供のコンテンツには絶対に使用しないでください。

サニタイズライブラリと組み合わせて使用することを強く推奨します。`,
    examples: [
      `<div v-html="rawHtmlContent"></div>`,
      `<!-- サニタイズと組み合わせ -->
<div v-html="sanitize(userContent)"></div>`
    ],
    docUrl: 'https://vuejs.org/api/built-in-directives.html#v-html',
    since: 'Vue 2.0'
  },

  'v-text': {
    signature: 'v-text="expression"',
    description: `**v-text** は要素の textContent を更新します。

mustache 補間 \`{{ }}\` と同等ですが、要素の内容全体を置き換えます。

部分的な更新が必要な場合は mustache 補間を使用してください。`,
    examples: [
      `<span v-text="message"></span>
<!-- 以下と同等 -->
<span>{{ message }}</span>`
    ],
    docUrl: 'https://vuejs.org/api/built-in-directives.html#v-text',
    since: 'Vue 2.0'
  }
};

// Helper function to format hover content as Markdown
function formatHoverContent(doc: HoverDoc): string {
  let content = '';

  // Signature (code block)
  content += '```typescript\n' + doc.signature + '\n```\n\n';

  // Description
  content += doc.description + '\n\n';

  // Examples
  if (doc.examples.length > 0) {
    content += '---\n\n**Examples:**\n\n';
    for (const example of doc.examples) {
      content += '```typescript\n' + example + '\n```\n\n';
    }
  }

  // Metadata
  const metadata: string[] = [];
  if (doc.since) metadata.push(`**Since:** ${doc.since}`);
  if (doc.deprecated) metadata.push(`**Deprecated:** ${doc.deprecated}`);
  if (doc.seeAlso && doc.seeAlso.length > 0) {
    metadata.push(`**See also:** ${doc.seeAlso.map(s => '`' + s + '`').join(', ')}`);
  }

  if (metadata.length > 0) {
    content += '---\n\n' + metadata.join(' | ') + '\n\n';
  }

  // Documentation link
  content += `[📖 Vue.js Documentation](${doc.docUrl})`;

  return content;
}

export interface Diagnostic {
  message: string;
  startLine: number;
  startColumn: number;
  endLine?: number;
  endColumn?: number;
  severity: 'error' | 'warning' | 'info';
}

const props = defineProps<{
  modelValue: string;
  language: string;
  diagnostics?: Diagnostic[];
}>();

const emit = defineEmits<{
  (e: 'update:modelValue', value: string): void;
}>();

const containerRef = ref<HTMLDivElement | null>(null);
const editorInstance = shallowRef<monaco.editor.IStandaloneCodeEditor | null>(null);
let isConfigured = false;

function configureMonaco() {
  if (isConfigured) return;
  isConfigured = true;

  // Register Vue language
  monaco.languages.register({ id: 'vue', extensions: ['.vue'] });

  // Set monarch tokenizer for Vue (HTML-based with Vue extensions)
  monaco.languages.setMonarchTokensProvider('vue', {
    defaultToken: '',
    tokenPostfix: '.vue',
    keywords: ['v-if', 'v-else', 'v-else-if', 'v-for', 'v-show', 'v-model', 'v-bind', 'v-on', 'v-slot', 'v-pre', 'v-once', 'v-memo', 'v-cloak'],
    tokenizer: {
      root: [
        [/<!--/, { token: 'comment', next: '@htmlComment' }],
        [/<script\s+setup\s+vapor[^>]*>/, { token: 'tag', next: '@script' }],
        [/<script\s+setup[^>]*>/, { token: 'tag', next: '@script' }],
        [/<script[^>]*>/, { token: 'tag', next: '@script' }],
        [/<style[^>]*>/, { token: 'tag', next: '@style' }],
        [/<template[^>]*>/, { token: 'tag', next: '@template' }],
        [/<\/?[\w-]+/, { token: 'tag', next: '@tag' }],
        [/\{\{/, { token: 'delimiter.bracket', next: '@interpolation' }],
      ],
      tag: [
        [/\s+/, ''],
        [/(v-[\w-]+|@[\w.-]+|:[\w.-]+|#[\w.-]+)/, 'attribute.name.vue'],
        [/[\w-]+/, 'attribute.name'],
        [/=/, 'delimiter'],
        [/"[^"]*"/, 'attribute.value'],
        [/'[^']*'/, 'attribute.value'],
        [/>/, { token: 'tag', next: '@pop' }],
        [/\/>/, { token: 'tag', next: '@pop' }],
      ],
      template: [
        [/<\/template>/, { token: 'tag', next: '@pop' }],
        [/<!--/, { token: 'comment', next: '@htmlComment' }],
        [/\{\{/, { token: 'delimiter.bracket', next: '@interpolation' }],
        [/<\/?[\w-]+/, { token: 'tag', next: '@tag' }],
        [/./, ''],
      ],
      htmlComment: [
        [/-->/, { token: 'comment', next: '@pop' }],
        [/./, 'comment'],
      ],
      interpolation: [
        [/\}\}/, { token: 'delimiter.bracket', next: '@pop' }],
        [/[\w.]+/, 'variable'],
        [/./, ''],
      ],
      script: [
        [/<\/script>/, { token: 'tag', next: '@pop' }],
        [/(import|export|from|const|let|var|function|return|if|else|for|while|class|interface|type|extends|implements)(?=\s)/, 'keyword'],
        [/(defineProps|defineEmits|defineExpose|defineOptions|defineSlots|defineModel|withDefaults)/, 'keyword.control.vue'],
        [/(ref|reactive|computed|watch|watchEffect|onMounted|onUnmounted|toRef|toRefs)/, 'support.function.vue'],
        [/"[^"]*"/, 'string'],
        [/'[^']*'/, 'string'],
        [/`[^`]*`/, 'string'],
        [/\/\/.*$/, 'comment'],
        [/\/\*/, { token: 'comment', next: '@comment' }],
        [/[{}()[\]]/, 'delimiter.bracket'],
        [/[<>]=?|[!=]=?=?|&&|\|\|/, 'operator'],
        [/\d+/, 'number'],
        [/[\w$]+/, 'identifier'],
        [/./, ''],
      ],
      comment: [
        [/\*\//, { token: 'comment', next: '@pop' }],
        [/./, 'comment'],
      ],
      style: [
        [/<\/style>/, { token: 'tag', next: '@pop' }],
        [/\/\*/, { token: 'comment', next: '@cssComment' }],
        [/[\w-]+(?=\s*:)/, 'attribute.name'],
        [/:/, 'delimiter'],
        [/[{}]/, 'delimiter.bracket'],
        [/"[^"]*"/, 'string'],
        [/'[^']*'/, 'string'],
        [/#[\da-fA-F]+/, 'number.hex'],
        [/\d+[\w%]*/, 'number'],
        [/[\w-]+/, 'attribute.value'],
        [/./, ''],
      ],
      cssComment: [
        [/\*\//, { token: 'comment', next: '@pop' }],
        [/./, 'comment'],
      ],
    },
  });

  // Set Vue language configuration
  monaco.languages.setLanguageConfiguration('vue', {
    comments: {
      blockComment: ['<!--', '-->'],
    },
    brackets: [
      ['<!--', '-->'],
      ['<', '>'],
      ['{', '}'],
      ['[', ']'],
      ['(', ')'],
    ],
    autoClosingPairs: [
      { open: '{', close: '}' },
      { open: '[', close: ']' },
      { open: '(', close: ')' },
      { open: '"', close: '"' },
      { open: "'", close: "'" },
      { open: '`', close: '`' },
      { open: '<', close: '>' },
      { open: '<!--', close: '-->' },
    ],
    surroundingPairs: [
      { open: '{', close: '}' },
      { open: '[', close: ']' },
      { open: '(', close: ')' },
      { open: '"', close: '"' },
      { open: "'", close: "'" },
      { open: '<', close: '>' },
    ],
  });

  // Register completion provider for Vue compiler macros and reactivity APIs
  monaco.languages.registerCompletionItemProvider('vue', {
    triggerCharacters: ['d', 'r', 'c', 'w', 't'],
    provideCompletionItems: (model, position) => {
      const word = model.getWordUntilPosition(position);
      const range = {
        startLineNumber: position.lineNumber,
        endLineNumber: position.lineNumber,
        startColumn: word.startColumn,
        endColumn: word.endColumn,
      };

      const textUntilPosition = model.getValueInRange({
        startLineNumber: 1,
        startColumn: 1,
        endLineNumber: position.lineNumber,
        endColumn: position.column,
      });

      const isInScriptSetup = /<script[^>]*setup[^>]*>/.test(textUntilPosition) &&
        !/<\/script>/.test(textUntilPosition.split(/<script[^>]*setup[^>]*>/)[1] || '');

      if (!isInScriptSetup) {
        return { suggestions: [] };
      }

      const suggestions = [
        ...VUE_COMPILER_MACROS.map(macro => ({
          label: macro.label,
          kind: monaco.languages.CompletionItemKind.Function,
          insertText: macro.insertText,
          insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
          detail: macro.detail,
          range,
        })),
        ...VUE_REACTIVITY_APIS.map(api => ({
          label: api.label,
          kind: monaco.languages.CompletionItemKind.Function,
          insertText: api.insertText,
          insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
          detail: api.detail,
          range,
        })),
      ];

      return { suggestions };
    },
  });

  // Register hover provider for Vue APIs
  monaco.languages.registerHoverProvider('vue', {
    provideHover: (model, position) => {
      const lineContent = model.getLineContent(position.lineNumber);

      // Check for directives (v-if, v-for, @click, :class, etc.)
      // Look for directive patterns around the cursor position
      const directivePatterns = [
        // v-directive pattern
        { regex: /v-[\w-]+/g, prefix: '' },
        // @ shorthand for v-on
        { regex: /@[\w.-]+/g, prefix: 'v-on' },
        // : shorthand for v-bind
        { regex: /:[\w.-]+/g, prefix: 'v-bind' },
        // # shorthand for v-slot
        { regex: /#[\w.-]+/g, prefix: 'v-slot' },
      ];

      for (const { regex, prefix } of directivePatterns) {
        let match;
        while ((match = regex.exec(lineContent)) !== null) {
          const startCol = match.index + 1;
          const endCol = startCol + match[0].length;

          if (position.column >= startCol && position.column <= endCol) {
            let directiveName = match[0];

            // Handle shorthands
            if (directiveName.startsWith('@')) {
              directiveName = 'v-on';
            } else if (directiveName.startsWith(':')) {
              directiveName = 'v-bind';
            } else if (directiveName.startsWith('#')) {
              directiveName = 'v-slot';
            } else {
              // Extract base directive name (e.g., v-on:click -> v-on)
              directiveName = directiveName.split(':')[0].split('.')[0];
            }

            const directiveDoc = VUE_DIRECTIVE_DOCS[directiveName];
            if (directiveDoc) {
              return {
                range: {
                  startLineNumber: position.lineNumber,
                  endLineNumber: position.lineNumber,
                  startColumn: startCol,
                  endColumn: endCol,
                },
                contents: [
                  { value: formatHoverContent(directiveDoc) }
                ],
              };
            }
          }
        }
      }

      const word = model.getWordAtPosition(position);
      if (!word) return null;

      const wordText = word.word;

      // Check compiler macros
      const macroDoc = COMPILER_MACRO_DOCS[wordText];
      if (macroDoc) {
        return {
          range: {
            startLineNumber: position.lineNumber,
            endLineNumber: position.lineNumber,
            startColumn: word.startColumn,
            endColumn: word.endColumn,
          },
          contents: [
            { value: formatHoverContent(macroDoc) }
          ],
        };
      }

      // Check Vue APIs
      const apiDoc = VUE_API_DOCS[wordText];
      if (apiDoc) {
        return {
          range: {
            startLineNumber: position.lineNumber,
            endLineNumber: position.lineNumber,
            startColumn: word.startColumn,
            endColumn: word.endColumn,
          },
          contents: [
            { value: formatHoverContent(apiDoc) }
          ],
        };
      }

      return null;
    },
  });

  // Register completion provider for SFC tag attributes
  monaco.languages.registerCompletionItemProvider('vue', {
    triggerCharacters: [' '],
    provideCompletionItems: (model, position) => {
      const lineContent = model.getLineContent(position.lineNumber);
      const textBeforeCursor = lineContent.substring(0, position.column - 1);

      const word = model.getWordUntilPosition(position);
      const range = {
        startLineNumber: position.lineNumber,
        endLineNumber: position.lineNumber,
        startColumn: word.startColumn,
        endColumn: word.endColumn,
      };

      const scriptTagMatch = textBeforeCursor.match(/<script\s+(?![^>]*>)/);
      const templateTagMatch = textBeforeCursor.match(/<template\s+(?![^>]*>)/);
      const styleTagMatch = textBeforeCursor.match(/<style\s+(?![^>]*>)/);

      let attrs: typeof SCRIPT_TAG_ATTRS = [];

      if (scriptTagMatch) {
        const usedAttrs: string[] = textBeforeCursor.match(/\b(setup|vapor|lang|generic)\b/g) || [];
        attrs = SCRIPT_TAG_ATTRS.filter(attr => {
          const attrName = attr.label.split('=')[0].split('"')[0];
          return !usedAttrs.includes(attrName);
        });
      } else if (templateTagMatch) {
        const usedAttrs: string[] = textBeforeCursor.match(/\b(lang)\b/g) || [];
        attrs = TEMPLATE_TAG_ATTRS.filter(attr => {
          const attrName = attr.label.split('=')[0].split('"')[0];
          return !usedAttrs.includes(attrName);
        });
      } else if (styleTagMatch) {
        const usedAttrs: string[] = textBeforeCursor.match(/\b(scoped|module|lang)\b/g) || [];
        attrs = STYLE_TAG_ATTRS.filter(attr => {
          const attrName = attr.label.split('=')[0].split('"')[0];
          return !usedAttrs.includes(attrName);
        });
      }

      if (attrs.length === 0) {
        return { suggestions: [] };
      }

      const suggestions = attrs.map(attr => ({
        label: attr.label,
        kind: monaco.languages.CompletionItemKind.Property,
        insertText: attr.insertText,
        insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
        detail: attr.detail,
        range,
      }));

      return { suggestions };
    },
  });

  // Define custom theme matching project CSS (Rust/Metal theme)
  monaco.editor.defineTheme('vue-dark', {
    base: 'vs-dark',
    inherit: true,
    rules: [
      { token: 'keyword', foreground: 'e07048' },
      { token: 'keyword.control.vue', foreground: 'f08060', fontStyle: 'bold' },
      { token: 'support.function.vue', foreground: 'e07048' },
      { token: 'attribute.name.vue', foreground: 'e07048' },
      { token: 'variable', foreground: 'd0d4dc' },
      { token: 'tag', foreground: 'e07048' },
      { token: 'attribute.name', foreground: '9ca3b0' },
      { token: 'attribute.value', foreground: 'd0d4dc' },
      { token: 'string', foreground: 'd0d4dc' },
      { token: 'number', foreground: 'f08060' },
      { token: 'comment', foreground: '6b7280' },
      { token: 'delimiter.bracket', foreground: '9ca3b0' },
      { token: 'identifier', foreground: 'f0f2f5' },
    ],
    colors: {
      'editor.background': '#1a1b21',
      'editor.foreground': '#f0f2f5',
      'editor.lineHighlightBackground': '#252830',
      'editor.selectionBackground': '#e0704840',
      'editorCursor.foreground': '#e07048',
      'editorLineNumber.foreground': '#6b7280',
      'editorLineNumber.activeForeground': '#9ca3b0',
      'editorIndentGuide.background': '#252830',
      'editorIndentGuide.activeBackground': '#e0704840',
      'editor.inactiveSelectionBackground': '#e0704820',
    },
  });
}

onMounted(() => {
  if (!containerRef.value) return;

  configureMonaco();

  editorInstance.value = monaco.editor.create(containerRef.value, {
    value: props.modelValue,
    language: props.language,
    theme: 'vue-dark',
    fontSize: 14,
    fontFamily: "'JetBrains Mono', monospace",
    minimap: { enabled: false },
    lineNumbers: 'on',
    scrollBeyondLastLine: false,
    padding: { top: 16 },
    automaticLayout: true,
    quickSuggestions: true,
    suggestOnTriggerCharacters: true,
  });

  editorInstance.value.onDidChangeModelContent(() => {
    const value = editorInstance.value?.getValue() || '';
    emit('update:modelValue', value);
  });
});

onUnmounted(() => {
  editorInstance.value?.dispose();
});

watch(() => props.modelValue, (newValue) => {
  if (editorInstance.value && editorInstance.value.getValue() !== newValue) {
    editorInstance.value.setValue(newValue);
  }
});

watch(() => props.language, (newLanguage) => {
  if (editorInstance.value) {
    const model = editorInstance.value.getModel();
    if (model) {
      monaco.editor.setModelLanguage(model, newLanguage);
    }
  }
});

// Update diagnostics markers
watch(() => props.diagnostics, (diagnostics) => {
  if (!editorInstance.value) return;
  const model = editorInstance.value.getModel();
  if (!model) return;

  if (!diagnostics || diagnostics.length === 0) {
    monaco.editor.setModelMarkers(model, 'vize', []);
    return;
  }

  const markers: monaco.editor.IMarkerData[] = diagnostics.map(d => ({
    severity: d.severity === 'error'
      ? monaco.MarkerSeverity.Error
      : d.severity === 'warning'
        ? monaco.MarkerSeverity.Warning
        : monaco.MarkerSeverity.Info,
    message: d.message,
    startLineNumber: d.startLine,
    startColumn: d.startColumn,
    endLineNumber: d.endLine ?? d.startLine,
    endColumn: d.endColumn ?? d.startColumn + 1,
  }));

  monaco.editor.setModelMarkers(model, 'vize', markers);
}, { immediate: true });
</script>

<template>
  <div ref="containerRef" class="monaco-container"></div>
</template>

<style scoped>
.monaco-container {
  width: 100%;
  height: 100%;
}
</style>
