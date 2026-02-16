<script setup lang="ts">
import type { Project } from '@/types/project';

const props = defineProps<{
  project: Project
}>();

const stats = [
  { title: 'Total Analysis', value: '1,284', trend: 12.5, icon: 'mdi-chart-bar', color: 'primary' },
  { title: 'Active Projects', value: '42', trend: 8.2, icon: 'mdi-folder-outline', color: 'success' },
  { title: 'AI Insights', value: '892', trend: -2.4, icon: 'mdi-auto-fix', color: 'warning' },
];
</script>

<template>
  <div class="dashboard-view">
    <div class="d-flex align-center justify-space-between mb-8">
      <div>
        <h1 class="text-h4 font-weight-bold mb-1">
          Overview: <span class="text-primary">{{ project.name }}</span>
        </h1>
        <p class="text-subtitle-1 text-grey">Key metrics and performance summary for {{ project.name }}</p>
      </div>
      <v-btn color="primary" prepend-icon="mdi-download">Export Report</v-btn>
    </div>

    <v-row>
      <v-col cols="12" md="4" v-for="(stat, i) in stats" :key="i">
        <v-card class="stat-card" elevation="2">
          <v-card-text class="d-flex align-center">
            <div :class="`icon-box bg-${stat.color}-lighten-5 mr-4`">
              <v-icon :icon="stat.icon" :color="stat.color" size="24" />
            </div>
            <div>
              <div class="text-caption text-grey mb-1 text-uppercase font-weight-bold">
                {{ stat.title }}
              </div>
              <div class="text-h5 font-weight-bold">{{ stat.value }}</div>
              <div class="text-caption" :class="stat.trend > 0 ? 'text-success' : 'text-error'">
                <v-icon :icon="stat.trend > 0 ? 'mdi-trending-up' : 'mdi-trending-down'" size="14" />
                {{ Math.abs(stat.trend) }}% from last month
              </div>
            </div>
          </v-card-text>
        </v-card>
      </v-col>
    </v-row>

    <div class="mt-8 text-center pa-12 border-dashed rounded-lg bg-white opacity-60">
      <v-icon icon="mdi-chart-arc" size="64" color="grey-lighten-1" class="mb-4" />
      <h3 class="text-h5 text-grey-darken-1 font-weight-bold">Dashboard content coming soon</h3>
      <p class="text-grey">We're building advanced analytics for your projects.</p>
    </div>
  </div>
</template>

<style scoped>
.stat-card {
  border-radius: 16px !important;
  border-bottom: 4px solid #6366f1;
}

.icon-box {
  width: 48px;
  height: 48px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.border-dashed {
  border: 2px dashed #e2e8f0;
}
</style>
