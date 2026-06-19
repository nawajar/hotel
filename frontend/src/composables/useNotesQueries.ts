import { useMutation, useQuery, useQueryClient } from "@tanstack/vue-query";
import { notesApi } from "@/api/notes";

const QUERY_KEY = ["notes"];

export function useNotesQuery() {
  return useQuery({
    queryKey: QUERY_KEY,
    queryFn: () => notesApi.list(),
  });
}

export function useCreateNoteMutation() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (body: string) => notesApi.create(body),
    onSuccess: () => qc.invalidateQueries({ queryKey: QUERY_KEY }),
  });
}

export function useToggleNoteMutation() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, done }: { id: string; done: boolean }) => notesApi.update(id, done),
    onSuccess: () => qc.invalidateQueries({ queryKey: QUERY_KEY }),
  });
}

export function useDeleteNoteMutation() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => notesApi.delete(id),
    onSuccess: () => qc.invalidateQueries({ queryKey: QUERY_KEY }),
  });
}
