<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { getBasePath } from '../api'

interface DesignToken {
  value: string | number
  type?: string
  description?: string
}

interface TokenCategory {
  name: string
  tokens: Record<string, DesignToken>
  subcategories?: TokenCategory[]
}

const categories = ref<TokenCategory[]>([])
const loading = ref(false)
const error = ref<string | null>(null)
const filter = ref('')

const filteredCategories = computed(() => {
  if (!filter.value) return categories.value
  const q = filter.value.toLowerCase()
  return categories.value
    .map(cat => filterCategory(cat, q))
    .filter((cat): cat is TokenCategory => cat !== null)
})

function filterCategory(cat: TokenCategory, query: string): TokenCategory | null {
  const matchingTokens: Record<string, DesignToken> = {}
  for (const [name, token] of Object.entries(cat.tokens)) {
    if (
      name.toLowerCase().includes(query) ||
      String(token.value).toLowerCase().includes(query) ||
      (token.description ?? '').toLowerCase().includes(query)
    ) {
      matchingTokens[name] = token
    }
  }

  const matchingSubs = (cat.subcategories ?? [])
    .map(sub => filterCategory(sub, query))
    .filter((sub): sub is TokenCategory => sub !== null)

  if (Object.keys(matchingTokens).length > 0 || matchingSubs.length > 0) {
    return {
      name: cat.name,
      tokens: matchingTokens,
      subcategories: matchingSubs.length > 0 ? matchingSubs : undefined,
    }
  }

  return null
}

function isColor(token: DesignToken): boolean {
  if (token.type === 'color') return true
  if (typeof token.value !== 'string') return false
  return token.value.startsWith('#') || token.value.startsWith('rgb') || token.value.startsWith('hsl')
}

onMounted(async () => {
  loading.value = true
  try {
    const res = await fetch(getBasePath() + '/api/tokens')
    if (res.ok) {
      const data = await res.json()
      categories.value = data.categories ?? data ?? []
    } else {
      error.value = 'No design tokens configured. Add a Style Dictionary config to see tokens here.'
    }
  } catch {
    error.value = 'No design tokens configured. Add a Style Dictionary config to see tokens here.'
  } finally {
    loading.value = false
  }
})
</script>

<template>
  <div class="tokens-view">
    <div class="tokens-header">
      <h1 class="tokens-title">Design Tokens</h1>
      <p class="tokens-description">
        Browse design tokens from your Style Dictionary configuration
      </p>
      <input
        v-model="filter"
        type="text"
        class="tokens-filter"
        placeholder="Filter tokens..."
      >
    </div>

    <div v-if="loading" class="tokens-loading">
      <div class="loading-spinner" />
      Loading tokens...
    </div>

    <div v-else-if="error" class="tokens-empty">
      <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <circle cx="12" cy="12" r="5" />
        <line x1="12" y1="1" x2="12" y2="3" />
        <line x1="12" y1="21" x2="12" y2="23" />
        <line x1="4.22" y1="4.22" x2="5.64" y2="5.64" />
        <line x1="18.36" y1="18.36" x2="19.78" y2="19.78" />
        <line x1="1" y1="12" x2="3" y2="12" />
        <line x1="21" y1="12" x2="23" y2="12" />
      </svg>
      <p>{{ error }}</p>
    </div>

    <template v-else>
      <div
        v-for="cat in filteredCategories"
        :key="cat.name"
        class="token-category"
      >
        <h2 class="category-title">{{ cat.name }}</h2>
        <div class="tokens-grid">
          <div
            v-for="(token, name) in cat.tokens"
            :key="name"
            class="token-card"
          >
            <div class="token-preview">
              <div
                v-if="isColor(token)"
                class="color-swatch"
                :style="{ background: String(token.value) }"
              />
            </div>
            <div class="token-info">
              <div class="token-name">{{ name }}</div>
              <div class="token-value">{{ token.value }}</div>
              <div v-if="token.description" class="token-desc">{{ token.description }}</div>
            </div>
          </div>
        </div>

        <template v-if="cat.subcategories">
          <div
            v-for="sub in cat.subcategories"
            :key="sub.name"
            class="token-subcategory"
          >
            <h3 class="subcategory-title">{{ sub.name }}</h3>
            <div class="tokens-grid">
              <div
                v-for="(token, name) in sub.tokens"
                :key="name"
                class="token-card"
              >
                <div class="token-preview">
                  <div
                    v-if="isColor(token)"
                    class="color-swatch"
                    :style="{ background: String(token.value) }"
                  />
                </div>
                <div class="token-info">
                  <div class="token-name">{{ name }}</div>
                  <div class="token-value">{{ token.value }}</div>
                  <div v-if="token.description" class="token-desc">{{ token.description }}</div>
                </div>
              </div>
            </div>
          </div>
        </template>
      </div>
    </template>
  </div>
</template>

<style scoped>
.tokens-view {
  max-width: 1400px;
  margin: 0 auto;
  padding: 2rem;
}

.tokens-header {
  margin-bottom: 2rem;
}

.tokens-title {
  font-size: 1.5rem;
  font-weight: 700;
  margin-bottom: 0.5rem;
}

.tokens-description {
  color: var(--musea-text-muted);
  font-size: 0.9375rem;
  margin-bottom: 1rem;
}

.tokens-filter {
  width: 100%;
  max-width: 400px;
  background: var(--musea-bg-secondary);
  border: 1px solid var(--musea-border);
  border-radius: var(--musea-radius-md);
  padding: 0.5rem 0.75rem;
  color: var(--musea-text);
  font-size: 0.8125rem;
  outline: none;
  transition: border-color var(--musea-transition);
}

.tokens-filter:focus {
  border-color: var(--musea-accent);
}

.tokens-loading {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  justify-content: center;
  min-height: 200px;
  color: var(--musea-text-muted);
}

.loading-spinner {
  width: 20px;
  height: 20px;
  border: 2px solid var(--musea-border);
  border-top-color: var(--musea-accent);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.tokens-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  min-height: 300px;
  text-align: center;
  color: var(--musea-text-muted);
  gap: 1rem;
}

.token-category {
  margin-bottom: 2.5rem;
}

.category-title {
  font-size: 1.125rem;
  font-weight: 600;
  margin-bottom: 1rem;
  padding-bottom: 0.5rem;
  border-bottom: 1px solid var(--musea-border);
}

.token-subcategory {
  margin-top: 1.5rem;
  margin-left: 1rem;
}

.subcategory-title {
  font-size: 0.9375rem;
  font-weight: 600;
  margin-bottom: 0.75rem;
  color: var(--musea-text-secondary);
}

.tokens-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 1rem;
}

.token-card {
  background: var(--musea-bg-secondary);
  border: 1px solid var(--musea-border);
  border-radius: var(--musea-radius-md);
  padding: 1rem;
  display: flex;
  gap: 1rem;
  align-items: center;
  transition: border-color var(--musea-transition);
}

.token-card:hover {
  border-color: var(--musea-text-muted);
}

.token-preview {
  flex-shrink: 0;
  width: 48px;
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.color-swatch {
  width: 48px;
  height: 48px;
  border-radius: var(--musea-radius-md);
  border: 1px solid var(--musea-border);
}

.token-info {
  flex: 1;
  min-width: 0;
}

.token-name {
  font-weight: 600;
  font-family: var(--musea-font-mono);
  font-size: 0.875rem;
  word-break: break-all;
}

.token-value {
  color: var(--musea-text-muted);
  font-family: var(--musea-font-mono);
  font-size: 0.75rem;
  word-break: break-all;
}

.token-desc {
  color: var(--musea-text-muted);
  font-size: 0.75rem;
  margin-top: 0.25rem;
}
</style>
