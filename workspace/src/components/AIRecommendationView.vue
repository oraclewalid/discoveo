<script setup lang="ts">
import type { Project } from '@/types/project';

const props = defineProps<{
  project: Project
}>();
</script>

<template>
  <div class="ai-view">
    <div class="mb-8">
      <h1 class="text-h4 font-weight-bold mb-1">Recommendations</h1>
      <p class="text-subtitle-1 text-grey">Prioritized insights and suggested actions for {{ project.name }}</p>
    </div>

    <v-card class="premium-ai-card pa-12 text-center" elevation="4">
      <div class="ai-sparkle mb-6">
        <v-icon icon="mdi-sparkles" size="80" color="white" />
      </div>
      
      <h2 class="text-h3 font-weight-bold text-white mb-4">The Magic is Brewing</h2>
      <p class="text-h6 text-white-70 mb-8 mx-auto" style="max-width: 600px">
        Our AI engine is processing your data to generate personalized recommendations. 
        You'll see smart suggestions here very soon.
      </p>

      <v-btn
        color="white"
        variant="flat"
        size="x-large"
        prepend-icon="mdi-lightning-bolt"
        class="magic-btn"
      >
        Notify Me When Ready
      </v-btn>
    </v-card>

    <v-row class="mt-8">
      <v-col cols="12" md="6" v-for="i in 2" :key="i">
        <v-card class="skeleton-card pa-6" elevation="0">
          <div class="d-flex align-center mb-4">
            <v-skeleton-loader type="avatar" class="mr-4" />
            <v-skeleton-loader type="text" width="60%" />
          </div>
          <v-skeleton-loader type="paragraph" />
        </v-card>
      </v-col>
    </v-row>
  </div>
</template>

<style scoped>
.premium-ai-card {
  background: linear-gradient(135deg, #6366f1 0%, #a855f7 50%, #ec4899 100%) !important;
  border-radius: 32px !important;
  position: relative;
  overflow: hidden;
}

.premium-ai-card::before {
  content: '';
  position: absolute;
  top: -50%;
  left: -50%;
  width: 200%;
  height: 200%;
  background: radial-gradient(circle, rgba(255,255,255,0.1) 0%, transparent 70%);
  animation: rotate 20s linear infinite;
}

@keyframes rotate {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.ai-sparkle {
  background: rgba(255, 255, 255, 0.2);
  display: inline-flex;
  padding: 30px;
  border-radius: 50%;
  backdrop-filter: blur(5px);
}

.text-white-70 {
  color: rgba(255, 255, 255, 0.7);
}

.magic-btn {
  border-radius: 16px !important;
  font-weight: 800;
  color: #6366f1 !important;
}

.skeleton-card {
  border: 1px solid #e2e8f0;
  border-radius: 20px !important;
}
</style>
