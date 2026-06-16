import { useMutation, useQuery, useQueryClient } from "@tanstack/vue-query";
import { usersApi, type CreateUserInput } from "@/api/users";

export function useUsersQuery() {
  return useQuery({
    queryKey: ["admin-users"],
    queryFn: usersApi.list,
  });
}

export function useCreateUserMutation() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (input: CreateUserInput) => usersApi.create(input),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["admin-users"] });
    },
  });
}
