<template>
  <div class="project-form">
    <h2>{{ isEditing ? 'Edit Project' : 'Create Project' }}</h2>

    <form @submit.prevent="handleSubmit">
      <div class="form-group">
        <label for="name">Project Name *</label>
        <input
          id="name"
          v-model="form.name"
          type="text"
          placeholder="Enter project name"
          required
          @input="validateName"
        />
        <span v-if="errors.name" class="error">{{ errors.name }}</span>
      </div>

      <div class="form-group">
        <label for="description">Description</label>
        <textarea
          id="description"
          v-model="form.description"
          placeholder="Enter project description"
          rows="4"
        />
      </div>

      <div class="form-actions">
        <button type="submit" :disabled="isLoading" class="btn btn-primary">
          {{ isLoading ? 'Saving...' : isEditing ? 'Update Project' : 'Create Project' }}
        </button>
        <button type="button" @click="handleCancel" class="btn btn-secondary">
          Cancel
        </button>
      </div>

      <div v-if="apiError" class="error-message">
        {{ apiError }}
      </div>
    </form>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, watch } from 'vue';
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

const isLoading = ref(false);
const apiError = ref('');

const form = reactive({
  name: props.project?.name || '',
  description: props.project?.description || '',
});

const errors = reactive({
  name: '',
});

const validateName = () => {
  errors.name = form.name.trim().length === 0 ? 'Project name is required' : '';
};

const handleSubmit = async () => {
  validateName();
  if (errors.name) return;

  isLoading.value = true;
  apiError.value = '';

  try {
    const payload: CreateProjectDTO = {
      name: form.name,
      description: form.description,
      status: form.status,
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
    resetForm();
  } catch (error) {
    apiError.value = error instanceof Error ? error.message : 'An error occurred';
  } finally {
    isLoading.value = false;
  }
};

const handleCancel = () => {
  resetForm();
  emit('cancel');
};

const resetForm = () => {
  form.name = props.project?.name || '';
  form.description = props.project?.description || '';
  errors.name = '';
  apiError.value = '';
};

watch(() => props.project, () => {
  resetForm();
});
</script>

<style scoped>
.project-form {
  background: #f9f9f9;
  padding: 24px;
  border-radius: 8px;
  max-width: 500px;
  margin: 0 auto;
}

.project-form h2 {
  margin-bottom: 24px;
  color: #333;
  font-size: 20px;
}

.form-group {
  margin-bottom: 16px;
}

.form-group label {
  display: block;
  margin-bottom: 6px;
  font-weight: 500;
  color: #555;
  font-size: 14px;
}

.form-group input,
.form-group textarea,
.form-group select {
  width: 100%;
  padding: 10px;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 14px;
  font-family: inherit;
}

.form-group input:focus,
.form-group textarea:focus,
.form-group select:focus {
  outline: none;
  border-color: #4a90e2;
  box-shadow: 0 0 0 3px rgba(74, 144, 226, 0.1);
}

.form-group textarea {
  resize: vertical;
}

.form-actions {
  display: flex;
  gap: 12px;
  margin-top: 24px;
}

.btn {
  padding: 10px 20px;
  border: none;
  border-radius: 4px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.3s ease;
  flex: 1;
}

.btn-primary {
  background-color: #4a90e2;
  color: white;
}

.btn-primary:hover:not(:disabled) {
  background-color: #357abd;
}

.btn-primary:disabled {
  background-color: #ccc;
  cursor: not-allowed;
}

.btn-secondary {
  background-color: #e0e0e0;
  color: #333;
}

.btn-secondary:hover {
  background-color: #d0d0d0;
}

.error {
  display: block;
  color: #e74c3c;
  font-size: 12px;
  margin-top: 4px;
}

.error-message {
  padding: 12px;
  background-color: #ffe0e0;
  color: #e74c3c;
  border-radius: 4px;
  margin-top: 16px;
  font-size: 14px;
}
</style>
