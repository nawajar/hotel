import { useMutation, useQuery, useQueryClient } from "@tanstack/vue-query";
import { authApi, type LoginInput } from "@/api/auth";
import { useAuthStore } from "@/stores/auth";

export function useMeQuery() {
  return useQuery({
    queryKey: ["me"],
    queryFn: authApi.me,
    retry: false,
  });
}

export function useLoginMutation() {
  const queryClient = useQueryClient();
  const authStore = useAuthStore();

  return useMutation({
    mutationFn: (input: LoginInput) => authApi.login(input),
    onSuccess: (user) => {
      authStore.user = user;
      queryClient.invalidateQueries({ queryKey: ["me"] });
    },
  });
}
