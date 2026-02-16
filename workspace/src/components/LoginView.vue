<template>
  <v-container fluid class="fill-height login-container">
    <v-row no-gutters align="center" justify="center">
      <v-col cols="12" sm="8" md="6" lg="4">
        <v-card class="login-card pa-8" elevation="12">
          <div class="text-center mb-8">
            <v-icon icon="mdi-rocket-launch" size="64" color="primary" class="mb-4" />
            <h1 class="text-h4 font-weight-bold mb-2">Discoveo</h1>
            <p class="text-subtitle-1 text-grey-darken-1">Modern SaaS Analysis Platform</p>
          </div>

          <v-form @submit.prevent="handleLogin">
            <v-text-field
              v-model="email"
              label="Email Address"
              prepend-inner-icon="mdi-email-outline"
              type="email"
              required
              class="mb-4"
            />

            <v-text-field
              v-model="password"
              label="Password"
              prepend-inner-icon="mdi-lock-outline"
              type="password"
              required
              class="mb-6"
            />

            <v-btn
              type="submit"
              color="primary"
              size="x-large"
              block
              class="login-btn py-4"
              :loading="loading"
            >
              Get Started
            </v-btn>
          </v-form>

          <v-divider class="my-8">
            <span class="px-2 text-caption text-grey">OR CONTINUE WITH</span>
          </v-divider>

          <div class="d-flex justify-center gap-4">
            <v-btn icon="mdi-google" variant="outlined" color="grey-lighten-1" />
            <v-btn icon="mdi-github" variant="outlined" color="grey-lighten-1" />
            <v-btn icon="mdi-microsoft" variant="outlined" color="grey-lighten-1" />
          </div>

          <div class="text-center mt-8">
            <p class="text-body-2 text-grey">
              Don't have an account?
              <a href="#" class="text-primary font-weight-bold">Create Account</a>
            </p>
          </div>
        </v-card>
      </v-col>
    </v-row>
  </v-container>
</template>

<script setup lang="ts">
import { ref } from 'vue';

const emit = defineEmits(['login']);

const email = ref('admin@discoveo.ai');
const password = ref('password123');
const loading = ref(false);

const handleLogin = () => {
  loading.value = true;
  // Simulate API delay
  setTimeout(() => {
    loading.value = false;
    emit('login');
  }, 800);
};
</script>

<style scoped>
.login-container {
  background: linear-gradient(135deg, #f5f7fa 0%, #c3cfe2 100%);
  min-height: 100vh;
}

.login-card {
  border-radius: 24px !important;
  background: rgba(255, 255, 255, 0.9);
  backdrop-filter: blur(10px);
  border: 1px solid rgba(255, 255, 255, 0.4);
}

.login-btn {
  border-radius: 12px !important;
  font-weight: 700;
  letter-spacing: 0.5px;
  background: linear-gradient(135deg, #6366f1 0%, #8b5cf6 100%) !important;
  color: white !important;
  transition: all 0.3s ease;
}

.login-btn:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 15px rgba(99, 102, 241, 0.4);
}

.gap-4 {
  gap: 16px;
}
</style>
