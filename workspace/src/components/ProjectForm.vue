<template>
  <div class="project-form">
    <!-- Breadcrumb-style Back Navigation -->
    <div class="d-flex align-center mb-6">
      <v-btn
        @click="handleCancel"
        variant="text"
        color="grey-darken-1"
        prepend-icon="mdi-chevron-left"
        class="text-none px-0"
      >
        Discard and go back
      </v-btn>
    </div>

    <v-card class="premium-form-card" elevation="0">
      <v-card-text class="pa-10">
        <div class="header-section text-center mb-10">
          <v-avatar color="primary-lighten-5" size="80" class="mb-4">
            <v-icon :icon="isEditing ? 'mdi-pencil-box-outline' : 'mdi-plus-box-outline'" color="primary" size="40" />
          </v-avatar>
          <h1 class="text-h4 font-weight-bold tracking-tight mb-2">
            {{ isEditing ? 'Edit Project' : 'Create New Project' }}
          </h1>
          <p class="text-subtitle-1 text-grey-darken-1">
            {{ isEditing ? 'Update your project information and settings below.' : 'Launch a new analytics project by filling in the details below.' }}
          </p>
        </div>

        <v-form ref="formRef" @submit.prevent="handleSubmit">
          <div class="form-section mb-8">
            <div class="text-overline text-primary font-weight-bold mb-2">Basic Information</div>
            
            <v-label class="text-subtitle-2 font-weight-bold mb-2 text-grey-darken-3">Project Name</v-label>
            <v-text-field
              v-model="form.name"
              placeholder="e.g. Q1 Marketing Funnel"
              :rules="[rules.required]"
              variant="solo"
              flat
              density="comfortable"
              class="custom-field mb-6"
              bg-color="grey-lighten-5"
              required
            >
              <template v-slot:prepend-inner>
                <v-icon icon="mdi-rocket-launch-outline" size="20" color="primary" />
              </template>
            </v-text-field>

            <v-label class="text-subtitle-2 font-weight-bold mb-2 text-grey-darken-3">Description</v-label>
            <v-textarea
              v-model="form.description"
              placeholder="What is this project about? (Internal notes, objectives, etc.)"
              variant="solo"
              flat
              density="comfortable"
              class="custom-field mb-6"
              bg-color="grey-lighten-5"
              rows="4"
              auto-grow
            >
              <template v-slot:prepend-inner>
                <v-icon icon="mdi-text-box-outline" size="20" color="primary" />
              </template>
            </v-textarea>
          </div>

          <v-alert
            v-if="apiError"
            type="error"
            variant="tonal"
            closable
            @click:close="apiError = ''"
            class="mb-8 rounded-xl"
          >
            {{ apiError }}
          </v-alert>

          <v-divider class="my-8 opacity-10" />

          <div class="actions-section d-flex align-center justify-end gap-4">
            <v-btn
              color="grey-darken-1"
              variant="text"
              size="large"
              class="text-none font-weight-bold"
              @click="handleCancel"
              :disabled="isLoading"
            >
              Discard Changes
            </v-btn>
            <v-btn
              color="primary"
              variant="flat"
              size="large"
              class="text-none font-weight-bold px-10 elevation-4 rounded-xl"
              :loading="isLoading"
              @click="handleSubmit"
            >
              {{ isEditing ? 'Save Changes' : 'Create Project' }}
            </v-btn>
          </div>
        </v-form>
      </v-card-text>
    </v-card>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, watch } from 'vue';
import projectService from '@/services/projectService';
import type { Project, CreateProjectDTO } from '@/types/project';

interface Props {
  project?: Project;
  isEditing?: boolean;
}

interface Emits {
  (e: 'submit', project: Project): void;
  (e: 'cancel'): void;
}

const props = withDefaults(defineProps<Props>(), {
  isEditing: false,
});

const emit = defineEmits<Emits>();

const formRef = ref();
const isLoading = ref(false);
const apiError = ref('');

const form = reactive({
  name: props.project?.name || '',
  description: props.project?.description || '',
});

const rules = {
  required: (value: string) => !!value || 'Project name is required',
};

const handleSubmit = async () => {
  const { valid } = await formRef.value.validate();
  if (!valid) return;

  isLoading.value = true;
  apiError.value = '';

  try {
    const payload: CreateProjectDTO = {
      name: form.name,
      description: form.description,
    };

    let result: Project;

    if (props.isEditing && props.project) {
      result = await projectService.update({
        id: props.project.id,
        ...payload,
      });
    } else {
      result = await projectService.create(payload);
    }

    emit('submit', result);
  } catch (error) {
    apiError.value = error instanceof Error ? error.message : 'An error occurred';
  } finally {
    isLoading.value = false;
  }
};

const handleCancel = () => {
  emit('cancel');
};

watch(() => props.project, (newVal) => {
  if (newVal) {
    form.name = newVal.name;
    form.description = newVal.description || '';
  }
}, { immediate: true });
</script>

<style scoped>
.project-form {
  max-width: 800px;
  margin: 0 auto;
}

.premium-form-card {
  border-radius: 32px !important;
  border: 1px solid #e2e8f0 !important;
  background: white !important;
  overflow: hidden;
}

.custom-field :deep(.v-field) {
  border-radius: 16px !important;
  border: 1px solid #f1f5f9 !important;
  transition: all 0.2s ease;
}

.custom-field :deep(.v-field--focused) {
  border-color: #6366f1 !important;
  background: white !important;
  box-shadow: 0 0 0 4px rgba(99, 102, 241, 0.1) !important;
}

.tracking-tight {
  letter-spacing: -0.025em;
}

.gap-4 {
  gap: 16px;
}

.text-overline {
  letter-spacing: 0.1em !important;
}
</style>
