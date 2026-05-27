import { createRouter, createWebHistory } from 'vue-router'
import type { RouteRecordRaw } from 'vue-router'

const routes: RouteRecordRaw[] = [
  {
    path: '/',
    name: 'home',
    component: () => import('@/views/Home.vue')
  },
  {
    path: '/projects',
    name: 'projects',
    component: () => import('@/views/Projects.vue')
  },
  {
    path: '/plugins',
    name: 'plugins',
    component: () => import('@/views/Plugins.vue')
  },
  {
    path: '/engines',
    name: 'engines',
    component: () => import('@/views/Engines.vue')
  },
  {
    path: '/templates',
    name: 'templates',
    component: () => import('@/views/Templates.vue')
  },
  {
    path: '/build',
    name: 'build',
    component: () => import('@/views/Build.vue')
  },
  {
    path: '/about',
    name: 'about',
    component: () => import('@/views/About.vue')
  },
  {
    path: '/settings',
    name: 'settings',
    component: () => import('@/views/Settings.vue')
  },
  {
    path: '/updates',
    name: 'updates',
    component: () => import('@/views/Updates.vue')
  },
  {
    path: '/:pathMatch(.*)*',
    redirect: '/'
  }
]

const router = createRouter({
  history: createWebHistory(),
  routes
})

export default router
