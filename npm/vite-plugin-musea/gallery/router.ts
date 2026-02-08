import { createRouter, createWebHistory } from 'vue-router'
import HomeView from './views/HomeView.vue'
import ComponentView from './views/ComponentView.vue'
import TokensView from './views/TokensView.vue'

const basePath = (window as unknown as { __MUSEA_BASE_PATH__: string }).__MUSEA_BASE_PATH__ ?? '/__musea__'

export const router = createRouter({
  history: createWebHistory(basePath),
  routes: [
    {
      path: '/',
      name: 'home',
      component: HomeView,
    },
    {
      path: '/tokens',
      name: 'tokens',
      component: TokensView,
    },
    {
      path: '/component/:path(.*)',
      name: 'component',
      component: ComponentView,
    },
  ],
})
